// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use crate::{
    pal::{
        data_structures::{
            SockAddr,
            SockAddrIn,
            Socklen,
        },
        linux,
    },
    runtime::fail::Fail,
    scheduler::Yielder,
};
use ::std::{
    mem,
    net::SocketAddrV4,
    os::unix::prelude::RawFd,
};

/// This function polls connect on a socket file descriptor until the connection is established (or returns an error).
pub async fn connect_coroutine(fd: RawFd, addr: SocketAddrV4, yielder: Yielder) -> Result<(), Fail> {
    loop {
        let saddr: SockAddr = linux::socketaddrv4_to_sockaddr(&addr);
        match unsafe { libc::connect(fd, &saddr as *const SockAddr, mem::size_of::<SockAddrIn>() as Socklen) } {
            // Operation completed.
            stats if stats == 0 => {
                trace!("connection established ({:?})", addr);
                return Ok(());
            },

            // Operation not completed, thus parse errno to find out what happened.
            _ => {
                let errno: libc::c_int = unsafe { *libc::__errno_location() };

                // Operation in progress.
                if errno == libc::EINPROGRESS || errno == libc::EALREADY {
                    // Operation in progress. Check if cancelled.
                    match yielder.yield_once().await {
                        Ok(()) => continue,
                        Err(cause) => return Err(cause),
                    }
                }
                // Operation failed.
                else {
                    let message: String = format!("connect(): operation failed (errno={:?})", errno);
                    error!("{}", message);
                    return Err(Fail::new(errno, &message));
                }
            },
        }
    }
}
