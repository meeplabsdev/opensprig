use crate::{mutex_device::MutexDevice, types::Error};
use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    spi::{Async, Instance, Spi},
};
use embassy_sync::{
    blocking_mutex::Mutex as BlockingMutex, blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex,
};
use embassy_time::Delay;
use embedded_sdmmc::{Mode, RawDirectory, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};

const MAX_DEPTH: usize = 256;

#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

pub struct Storage<'a, T: Instance> {
    volume_manager: Mutex<
        ThreadModeRawMutex,
        VolumeManager<
            SdCard<MutexDevice<'a, Spi<'a, T, Async>, Output<'a>, Delay>, Delay>,
            DummyTimesource,
        >,
    >,
    root: RawDirectory,
}

impl<'a, T: Instance> Storage<'a, T> {
    pub async fn new(
        spi: &'a BlockingMutex<ThreadModeRawMutex, Spi<'a, T, Async>>,
        cs: Peri<'a, impl Pin>,
    ) -> Result<Self, Error> {
        let sd_device = MutexDevice::new(spi, Output::new(cs, Level::High), Delay).unwrap();
        let sd_card = SdCard::new(sd_device, Delay);

        let sd_size = sd_card.num_bytes()?;
        defmt::debug!("Card size: {} bytes", sd_size);

        let volume_manager = Mutex::new(VolumeManager::new(sd_card, DummyTimesource::default()));
        let root = volume_manager
            .lock()
            .await
            .open_volume(VolumeIdx(0))?
            .open_root_dir()?
            .to_raw_directory();

        let mut storage = Self {
            volume_manager,
            root,
        };

        storage.list("/").await?;
        if !storage.exists("sprig.1").await {
            storage
                .write("sprig.1", &[], Mode::ReadWriteCreateOrTruncate)
                .await?;
        }

        Ok(storage)
    }

    async fn resolve<'p>(
        &mut self,
        path: &'p str,
    ) -> Result<(Option<RawDirectory>, &'p str), Error> {
        let mut path = path.trim();
        if let Some(p) = path.strip_prefix("/") {
            path = p;
        }
        if let Some(p) = path.strip_suffix("/") {
            path = p;
        }

        if path.contains("./") {
            return Err(Error::new("Path cannot contain \"./\" or \"../\""));
        }

        let mgr = self.volume_manager.lock().await;
        let parts = path.split("/");
        let count = parts.clone().count();

        if count > MAX_DEPTH {
            return Err(Error::new("Path too long"));
        } else if count == 0 {
            return Err(Error::new("Invalid path"));
        } else if count == 1 {
            return Ok((None, parts.last().unwrap()));
        }

        let mut dirs: [Option<RawDirectory>; MAX_DEPTH] = [None; MAX_DEPTH];
        let mut name = ".";

        for (i, part) in parts.enumerate() {
            if i == count - 1 {
                name = part;
            } else {
                let dir = match i.checked_sub(1).map(|i| dirs.get(i)) {
                    Some(Some(Some(d))) => *d,
                    _ => self.root,
                };

                dirs[i] = match mgr.open_dir(dir, part) {
                    Ok(d) => Some(d),
                    Err(e) => return Err(e.into()),
                };

                defmt::debug!("Opened dir {}", dirs[i]);
            }
        }

        mgr.find_directory_entry(dirs[count - 2].unwrap(), name)?;

        for i in (0..count - 2).rev() {
            if dirs[i] == None {
                continue;
            }

            mgr.close_dir(dirs[i].unwrap())?;
        }

        Ok((*dirs.get(count - 2).unwrap_or(&None), name))
    }

    pub async fn exists(&mut self, path: &str) -> bool {
        let Ok((directory, name)) = self.resolve(path).await else {
            return false;
        };

        let mgr = self.volume_manager.lock().await;
        let dir = directory.unwrap_or(self.root);
        let found = mgr.find_directory_entry(dir, name).is_ok();

        if let Some(d) = directory {
            let _ = mgr.close_dir(d);
        }

        found
    }

    pub async fn list(&mut self, path: &str) -> Result<(), Error> {
        let (directory, name) = self.resolve(path).await?;

        let mgr = self.volume_manager.lock().await;
        let dir = mgr.open_dir(directory.unwrap_or(self.root), name)?;
        defmt::debug!("Opened {}", dir);

        mgr.iterate_dir(dir, |entry| {
            if entry.attributes.is_directory() {
                defmt::info!("{} <DIR>", str::from_utf8(entry.name.base_name()).unwrap());
            } else {
                defmt::info!(
                    "{}.{}",
                    str::from_utf8(entry.name.base_name()).unwrap(),
                    str::from_utf8(entry.name.extension()).unwrap()
                );
            }
        })?;

        mgr.close_dir(dir)?;
        if let Some(d) = directory {
            mgr.close_dir(d)?;
        }

        Ok(())
    }

    pub async fn write(&mut self, path: &str, buffer: &[u8], mode: Mode) -> Result<(), Error> {
        let (directory, name) = self.resolve(path).await?;

        let mgr = self.volume_manager.lock().await;
        let file = mgr.open_file_in_dir(directory.unwrap_or(self.root), name, mode)?;
        mgr.write(file, buffer)?;
        mgr.close_file(file)?;

        if let Some(d) = directory {
            mgr.close_dir(d)?;
        }

        Ok(())
    }

    pub async fn read<const S: usize>(
        &mut self,
        path: &str,
        offset: u32,
    ) -> Result<[u8; S], Error> {
        let (directory, name) = self.resolve(path).await?;
        let mgr = self.volume_manager.lock().await;

        let file = match mgr.open_file_in_dir(directory.unwrap_or(self.root), name, Mode::ReadOnly)
        {
            Ok(f) => f,
            Err(e) => {
                if let Some(d) = directory {
                    let _ = mgr.close_dir(d);
                }
                return Err(e.into());
            }
        };

        let mut buf = [0u8; S];
        let result: Result<(), Error> = (|| {
            mgr.file_seek_from_start(file, offset)?;
            mgr.read(file, &mut buf)?;
            Ok(())
        })();

        let _ = mgr.close_file(file);
        if let Some(d) = directory {
            let _ = mgr.close_dir(d);
        }

        result?;
        Ok(buf)
    }

    pub async fn read_into(
        &mut self,
        path: &str,
        buffer: &mut [u8],
        mut on_chunk: impl FnMut(u32, &[u8]) -> Result<(), Error>,
    ) -> Result<u32, Error> {
        let (directory, name) = self.resolve(path).await?;
        let mgr = self.volume_manager.lock().await;
        let dir = directory.unwrap_or(self.root);

        let file = match mgr.open_file_in_dir(dir, name, Mode::ReadOnly) {
            Ok(f) => f,
            Err(e) => {
                if let Some(d) = directory {
                    let _ = mgr.close_dir(d);
                }
                return Err(e.into());
            }
        };

        let mut offset: u32 = 0;
        let result: Result<(), Error> = (|| {
            loop {
                let read = mgr.read(file, buffer)?;
                if read == 0 {
                    break;
                }
                on_chunk(offset, &buffer[..read])?;
                offset += read as u32;
                if read < buffer.len() {
                    break;
                }
            }
            Ok(())
        })();

        let _ = mgr.close_file(file);
        if let Some(d) = directory {
            let _ = mgr.close_dir(d);
        }

        result?;
        Ok(offset)
    }

    pub async fn create(&mut self, path: &str) -> Result<(), Error> {
        self.write(path, &[], Mode::ReadWriteCreate).await
    }

    pub async fn delete(&mut self, path: &str) -> Result<(), Error> {
        let (directory, name) = self.resolve(path).await?;

        let mgr = self.volume_manager.lock().await;
        mgr.delete_file_in_dir(directory.unwrap_or(self.root), name)?;

        if let Some(d) = directory {
            mgr.close_dir(d)?;
        }

        Ok(())
    }

    pub async fn file_size(&mut self, path: &str) -> Result<u32, Error> {
        let (directory, name) = self.resolve(path).await?;
        let mgr = self.volume_manager.lock().await;
        let dir = directory.unwrap_or(self.root);

        let result = mgr.find_directory_entry(dir, name).map(|e| e.size);

        if let Some(d) = directory {
            let _ = mgr.close_dir(d);
        }

        Ok(result?)
    }
}

impl<'a, T: Instance> Drop for Storage<'a, T> {
    fn drop(&mut self) {
        let _ = self.volume_manager.get_mut().close_dir(self.root);
    }
}
