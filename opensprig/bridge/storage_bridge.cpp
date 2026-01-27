#include "storage.h"

extern "C" {
    FIL *File_new() {
        return new FIL();
    }

    void File_free(FIL *file) {
        delete file;
    }

    FRESULT File_raw_open(FIL *file, const char *path, unsigned char mode) {
        return f_open(file, path, mode);
    }

    FRESULT File_raw_close(FIL *file) {
        return f_close(file);
    }

    Storage *Storage_new() {
        return new Storage();
    }

    void Storage_free(Storage *storage) {
        delete storage;
    }

    bool Storage_is_ready(Storage *storage) {
        return storage->is_ready();
    }

    FRESULT Storage_mount(Storage *storage) {
        return storage->mount();
    }

    FRESULT Storage_unmount(Storage *storage) {
        return storage->unmount();
    }

    FRESULT Storage_raw_write(Storage *storage, FIL *file, char *text) {
        return storage->raw_write(file, text);
    }

    FRESULT Storage_write(Storage *storage, const char *path, char *text) {
        return storage->write(path, text);
    }

    FRESULT Storage_raw_read(Storage *storage, FIL *file, void *buffer,
                             uint64_t buffer_size) {
        return storage->raw_read(file, buffer, buffer_size);
    }

    FRESULT Storage_read(Storage *storage, const char *path, void *buffer,
                         uint64_t buffer_size, uint64_t offset) {
        return storage->read(path, buffer, buffer_size, offset);
    }
}
