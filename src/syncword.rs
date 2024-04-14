use crate::ringbuffer::RingBuffer;

pub struct SyncwordScanXCorr<S, const N: usize> {
    syncword: [S; N],
    error_hist: RingBuffer<u16, N>,
    error_thresh: u16,
}

impl<S, const N: usize> SyncwordScanXCorr<S, N>
where
    S: PartialEq,
{
    pub fn new(syncword: [S; N], error_thresh: u16) -> Self {
        Self {
            syncword,
            error_thresh,
            error_hist: RingBuffer::<u16, N>::with_init_val(error_thresh + 1),
        }
    }

    pub fn tick(&mut self, symbol_in: S) -> bool {
        self.error_hist.push(0);

        for idx in 0..N {
            if self.syncword[idx] != symbol_in {
                self.error_hist[-(idx as isize) - 1] += 1;
            }
        }

        self.error_hist[0usize] <= self.error_thresh
    }

    pub fn reset(&mut self) -> bool {
        for idx in 0..N {
            self.error_hist[idx] = self.error_thresh + 1;
        }
        self.error_hist[0usize] <= self.error_thresh
    }
}

pub struct SyncwordPacketizer<S, const SyncN: usize, const PacketN: usize> {
    scan: SyncwordScanXCorr<S, SyncN>,

    packet_active: bool,
    packet_buffer: [S; PacketN],
    packet_buffer_idx: usize,
}

impl<S, const SyncN: usize, const PacketN: usize> SyncwordPacketizer<S, SyncN, PacketN>
where
    S: PartialEq + Default + Copy,
{
    pub fn new(syncword: [S; SyncN], error_thresh: u16) -> Self {
        Self {
            scan: SyncwordScanXCorr::new(syncword, error_thresh),
            packet_active: false,
            packet_buffer: [S::default(); PacketN],
            packet_buffer_idx: 0,
        }
    }

    pub fn tick(&mut self, symbol_in: S) -> Option<&mut [S]> {
        if self.packet_active {
            self.packet_buffer[self.packet_buffer_idx] = symbol_in;
            self.packet_buffer_idx += 1;
            if self.packet_buffer_idx == PacketN {
                self.packet_active = false;
                self.packet_buffer_idx = 0;
                return Some(self.packet_buffer.as_mut_slice());
            }
            return None;
        }

        if self.scan.tick(symbol_in) {
            self.packet_active = true;
            self.scan.reset();
        }
        None
    }
}
