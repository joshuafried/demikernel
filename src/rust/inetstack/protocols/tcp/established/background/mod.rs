// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

mod acknowledger;
mod retransmitter;
mod sender;

use self::{
    acknowledger::acknowledger,
    retransmitter::retransmitter,
    sender::sender,
};
use super::ControlBlock;
use crate::runtime::QDesc;
use ::futures::{
    channel::mpsc,
    FutureExt,
};
use ::std::rc::Rc;

pub async fn background<const N: usize>(
    cb: Rc<ControlBlock<N>>,
    fd: QDesc,
    _dead_socket_tx: mpsc::UnboundedSender<QDesc>,
) {
    let acknowledger = acknowledger(cb.clone()).fuse();
    futures::pin_mut!(acknowledger);

    let retransmitter = retransmitter(cb.clone()).fuse();
    futures::pin_mut!(retransmitter);

    let sender = sender(cb.clone()).fuse();
    futures::pin_mut!(sender);

    let r = futures::select_biased! {
        r = acknowledger => r,
        r = retransmitter => r,
        r = sender => r,
    };
    error!("Connection (fd {:?}) terminated: {:?}", fd, r);

    // TODO Properly clean up Peer state for this connection.
    // dead_socket_tx
    //     .unbounded_send(fd)
    //     .expect("Failed to terminate connection");
}
