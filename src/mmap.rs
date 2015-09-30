use std;

use libc;

// =====================================

struct FileDescriptor(libc::c_int);

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        let FileDescriptor(fd) = *self;
        unsafe {
            libc::close(fd);
        }
    }
}

impl FileDescriptor {
    unsafe fn open(filename: &str) -> Result<FileDescriptor,String> {
        if let Ok(file) = std::ffi::CString::new(filename) {
            let fd = libc::open(file.as_ptr(), libc::O_RDONLY, 0);
            if fd < 0 {
                return Err( format!("failure in open({}): {}", filename, std::io::Error::last_os_error()) );
            } else {
                return Ok( FileDescriptor(fd) );
            }
        } else {
            return Err( format!("cannot convert filename to CString: {}", filename) );
        }
    }
}

#[test]
fn test_open_success() {
    unsafe {
        match FileDescriptor::open("Cargo.toml") {
            Ok(_) => { },
            Err(e) => { panic!(e); }
        }
    }
}

#[test]
fn test_open_failure() {
    unsafe {
        match FileDescriptor::open("nonexistent") {
            Ok(FileDescriptor(f)) => { panic!("open nonexistent file succeded: {}", f); },
            Err(_) =>  { }
        }
    }
}

// -------------------------------------

impl FileDescriptor {
    unsafe fn get_size(&self) -> Result<libc::size_t,String> {
        let FileDescriptor(fd) = *self;
        // I mean, really, WTF?
        let mut stat = libc::stat {
            st_dev:        0,
            st_ino:        0,
            st_nlink:      0,
            st_mode:       0,
            st_uid:        0,
            st_gid:        0,
            __pad0:        0,
            st_rdev:       0,
            st_size:       0,
            st_blksize:    0,
            st_blocks:     0,
            st_atime:      0,
            st_atime_nsec: 0,
            st_mtime:      0,
            st_mtime_nsec: 0,
            st_ctime:      0,
            st_ctime_nsec: 0,
            __unused:      [0; 3],
        };
        if libc::fstat(fd, &mut stat) < 0 {
            Err( format!("failure in fstat(): {}", std::io::Error::last_os_error()) )
        } else {
            Ok( stat.st_size as libc::size_t )
        }
    }

    fn get_fd(&self) -> libc::c_int {
        let FileDescriptor(fd) = *self;
        fd
    }
}

#[test]
fn test_get_size() {
    unsafe {
        let res = FileDescriptor::open("Cargo.toml")
            .and_then(|fd| { fd.get_size() })
            .and_then(|sz| { if sz == 134 { Ok(true) } else { Err(format!("{} != 134", sz)) }});
        if res.is_err() {
            panic!(res.unwrap_err());
        }
    }
}

// =====================================

pub struct MappedRegion {
    _fd: FileDescriptor,
    ptr: *mut u8,
    sz: libc::size_t,
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        unsafe {
            if libc::munmap(self.ptr as *mut libc::c_void, self.sz) < 0 {
                panic!("cannot munmap: {}", std::io::Error::last_os_error());
            }
        }
    }
}

impl MappedRegion {
    
    pub fn mmap(filename: &str) -> Result<MappedRegion,String> {
        unsafe {
            match FileDescriptor::open(filename) {
                Ok(fd) => {
                    map(fd)
                }
                Err(e) => { Err(e) }
            }
        }
    }

    pub fn get_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.sz as usize)
        }
    }

    pub fn get_str(&self) -> Result<&str,std::str::Utf8Error> {
        std::str::from_utf8(self.get_slice())
    }
}

unsafe fn map(fd: FileDescriptor) -> Result<MappedRegion,String> {
    match fd.get_size() {
        Ok(sz) => {
            let address = libc::mmap(0 as *mut libc::c_void, sz as u64, libc::PROT_READ, libc::MAP_PRIVATE, fd.get_fd(), 0);
            if address < 0 as *mut libc::c_void {
                Err( format!("failure in mmap(): {}", std::io::Error::last_os_error()) )
            } else {
                Ok( MappedRegion {
                    _fd: fd,
                    ptr: address as *mut u8,
                    sz: sz,
                })
            }
        }
        Err(e) => { Err(e) }
    }
}

#[test]
fn test_mmap() {
    match MappedRegion::mmap("Cargo.toml") {
        Ok(mr) => {
            assert_eq!(mr.get_slice().len(), 134);
            match mr.get_str() {
                Ok(s) => {
                    assert_eq!(s.lines_any().nth(0).unwrap(), "[package]");
                }
                Err(e) => { panic!(e); }
            }
        }
        Err(e) => { panic!(e); }
    }
}
