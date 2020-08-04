# file-locker

File locking via POSIX advisory record locks
(fork of [file-lock](https://gitlab.com/alfiedotwtf/file-lock))

This crate provides the facility to lock and unlock a file following the
advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001 (POSIX.1)
via fcntl().

This crate currently supports Linux and FreeBSD.

## USAGE

    use file_lock::FileLock;
    use std::io::prelude::*;
	use std::io::Result;

    fn main() -> Result<()> {
		let filelock = FileLock::new("myfile.txt")
						.writeable(true)
						.blocking(true)
						.lock()?;

        filelock.file.write_all(b"Hello, World!")?;

        // Manually unlocking is optional as we unlock on Drop
        filelock.unlock();
    }

## DOCUMENTATION

* [https://docs.rs/file-locker/](https://docs.rs/file-locker/)

## SUPPORT

Please report any bugs at:

* [https://todo.sr.ht/~zethra/file-locker](https://todo.sr.ht/~zethra/file-locker)

Or by email at:

* [~zethra/public-inbox@lists.sr.ht](mailto:~zethra/public-inbox@lists.sr.ht)

## AUTHORS

[Ben Goldberg](https://benaaron.dev) &lt;[benaagoldberg@gmail.com](mailto:benaagoldberg@gmail.com)&gt;

[Alfie John](https://www.alfie.wtf) &lt;[alfie@alfie.wtf](mailto:alfie@alfie.wtf)&gt;

[Sebastian Thiel](http://byronimo.de) &lt;[byronimo@gmail.com](mailto:byronimo@gmail.com)&gt;

## Contribution

Contribution welcome!

Please send any patches to [~zethra/public-inbox@lists.sr.ht](mailto:~zethra/public-inbox@lists.sr.ht)

If you need help sending a patch over email please see [this guide](https://git-send-email.io/)

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, shall be licensed under the
MIT license, without any additional terms or conditions.
