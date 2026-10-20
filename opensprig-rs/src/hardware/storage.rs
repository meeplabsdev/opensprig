use crate::{
    err,
    mutex_device::MutexDevice,
    slice_at_needle,
    types::{DebugError, Result},
};
use embassy_rp::{
    Peri,
    gpio::{Level, Output, Pin},
    spi::{Async, Instance, Spi},
};
use embassy_sync::{
    blocking_mutex::Mutex as BlockingMutex, blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex,
};
use embassy_time::Delay;
use embedded_sdmmc::{
    LfnBuffer, Mode, RawDirectory, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};

const MAX_DEPTH: usize = 256;
const MAX_ENTRIES: usize = 256;

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
    ) -> Result<Self> {
        let sd_device = MutexDevice::new(spi, Output::new(cs, Level::High), Delay).unwrap();
        let sd_card = SdCard::new(sd_device, Delay);

        let sd_size = sd_card.num_bytes().map_err(DebugError)?;
        defmt::debug!("Card size: {} bytes", sd_size);

        let volume_manager = Mutex::new(VolumeManager::new(sd_card, DummyTimesource::default()));
        let root = volume_manager
            .lock()
            .await
            .open_volume(VolumeIdx(0))
            .map_err(DebugError)?
            .open_root_dir()
            .map_err(DebugError)?
            .to_raw_directory();

        let storage = Self {
            volume_manager,
            root,
        };

        if !storage.exists("sprig.1").await {
            storage
                .write("sprig.1", &[], Mode::ReadWriteCreateOrTruncate)
                .await?;
        }

        Ok(storage)
    }

    async fn resolve<'p>(&self, path: &'p str) -> Result<(Option<RawDirectory>, &'p str)> {
        let mut path = path.trim();
        if let Some(p) = path.strip_prefix("/") {
            path = p;
        }
        if let Some(p) = path.strip_suffix("/") {
            path = p;
        }

        if path.contains("./") {
            return err!("Path cannot contain \"./\" or \"../\"");
        }

        let mgr = self.volume_manager.lock().await;
        let parts = path.split("/");
        let count = parts.clone().count();

        if count > MAX_DEPTH {
            return err!("Path too long");
        } else if count == 0 {
            return err!("Invalid path");
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
                    Err(e) => return Err(DebugError(e).into()),
                };

                defmt::debug!("Opened dir {}", dirs[i]);
            }
        }

        mgr.find_directory_entry(dirs[count - 2].unwrap(), name)
            .map_err(DebugError)?;

        for i in (0..count - 2).rev() {
            if dirs[i] == None {
                continue;
            }

            mgr.close_dir(dirs[i].unwrap()).map_err(DebugError)?;
        }

        Ok((*dirs.get(count - 2).unwrap_or(&None), name))
    }

    pub async fn exists(&self, path: &str) -> bool {
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

    pub async fn list(
        &self,
        path: &str,
        mut filter: impl FnMut(&[u8]) -> bool,
    ) -> Result<([[u8; 12]; MAX_ENTRIES], [[u8; 40]; MAX_ENTRIES])> {
        let (directory, name) = self.resolve(path).await?;

        let mgr = self.volume_manager.lock().await;
        let dir = mgr
            .open_dir(directory.unwrap_or(self.root), name)
            .map_err(DebugError)?;
        defmt::debug!("Opened {}", dir);

        let mut output = ([[0u8; 12]; MAX_ENTRIES], [[0u8; 40]; MAX_ENTRIES]);
        let mut i = 0;

        let mut lfn_storage = [0u8; 40];
        let mut lfn_buffer = LfnBuffer::new(&mut lfn_storage);
        mgr.iterate_dir_lfn(dir, &mut lfn_buffer, |entry, lfn| {
            let mut s_item = [0u8; 12];
            let mut l_item = [0u8; 40];

            let name = entry.name.base_name();
            let extension = entry.name.extension();

            if entry.attributes.is_directory() {
                s_item[..name.len()].copy_from_slice(name);
            } else {
                s_item[..name.len()].copy_from_slice(name);
                s_item[name.len()] = b'.';
                s_item[name.len() + 1..name.len() + extension.len() + 1].copy_from_slice(extension);
            }

            if let Some(name) = lfn {
                l_item[..name.len()].copy_from_slice(name.as_bytes());
            } else {
                l_item[..12].copy_from_slice(&s_item);
            }

            l_item.make_ascii_lowercase();
            if filter(slice_at_needle(&l_item, 0)) {
                output.0[i] = s_item;
                output.1[i] = l_item;
                i += 1;
            }
        })
        .map_err(DebugError)?;

        mgr.close_dir(dir).map_err(DebugError)?;
        if let Some(d) = directory {
            mgr.close_dir(d).map_err(DebugError)?;
        }

        Ok(output)
    }

    pub async fn write(&self, path: &str, buffer: &[u8], mode: Mode) -> Result<()> {
        let (directory, name) = self.resolve(path).await?;

        let mgr = self.volume_manager.lock().await;
        let file = mgr
            .open_file_in_dir(directory.unwrap_or(self.root), name, mode)
            .map_err(DebugError)?;
        mgr.write(file, buffer).map_err(DebugError)?;
        mgr.close_file(file).map_err(DebugError)?;

        if let Some(d) = directory {
            mgr.close_dir(d).map_err(DebugError)?;
        }

        Ok(())
    }

    pub async fn read<const S: usize>(&self, path: &str, offset: u32) -> Result<[u8; S]> {
        let (directory, name) = self.resolve(path).await?;
        let mgr = self.volume_manager.lock().await;

        let file = match mgr.open_file_in_dir(directory.unwrap_or(self.root), name, Mode::ReadOnly)
        {
            Ok(f) => f,
            Err(e) => {
                if let Some(d) = directory {
                    let _ = mgr.close_dir(d);
                }

                return Err(DebugError(e).into());
            }
        };

        let mut buf = [0u8; S];
        let result: Result<()> = (|| {
            mgr.file_seek_from_start(file, offset).map_err(DebugError)?;
            mgr.read(file, &mut buf).map_err(DebugError)?;
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
        &self,
        path: &str,
        buffer: &mut [u8],
        mut on_chunk: impl FnMut(u32, &[u8]) -> Result<()>,
    ) -> Result<u32> {
        let (directory, name) = self.resolve(path).await?;
        let mgr = self.volume_manager.lock().await;
        let dir = directory.unwrap_or(self.root);

        let file = match mgr.open_file_in_dir(dir, name, Mode::ReadOnly) {
            Ok(f) => f,
            Err(e) => {
                if let Some(d) = directory {
                    let _ = mgr.close_dir(d);
                }

                return Err(DebugError(e).into());
            }
        };

        let mut offset: u32 = 0;
        let result: Result<()> = (|| {
            loop {
                let read = mgr.read(file, buffer).map_err(DebugError)?;
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

    pub async fn create(&self, path: &str) -> Result<()> {
        self.write(path, &[], Mode::ReadWriteCreate).await
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let (directory, name) = self.resolve(path).await?;

        let mgr = self.volume_manager.lock().await;
        mgr.delete_file_in_dir(directory.unwrap_or(self.root), name)
            .map_err(DebugError)?;

        if let Some(d) = directory {
            mgr.close_dir(d).map_err(DebugError)?;
        }

        Ok(())
    }

    pub async fn file_size(&self, path: &str) -> Result<u32> {
        let (directory, name) = self.resolve(path).await?;
        let mgr = self.volume_manager.lock().await;
        let dir = directory.unwrap_or(self.root);

        let result = mgr.find_directory_entry(dir, name).map(|e| e.size);

        if let Some(d) = directory {
            let _ = mgr.close_dir(d);
        }

        Ok(result.map_err(DebugError)?)
    }
}

impl<'a, T: Instance> Drop for Storage<'a, T> {
    fn drop(&mut self) {
        let _ = self.volume_manager.get_mut().close_dir(self.root);
    }
}
