use std::time::Instant;
use crate::valgrind::MemoryAccess;

#[derive(Debug)]
pub struct Cache {
    pub stats: Statistics,
    sets: Box<[Set]>,
    set_bits: u8,
    block_bits: u8,
    tag_bits: u8,
    num_lines: u8,
}

#[derive(Debug)]
struct Set {
    lines: Box<[Line]>,
}

#[derive(Debug)]
struct Line {
    valid: bool,
    tag: u64,
    block: Box<[u8]>,
    access_time: Instant,
}

#[derive(PartialEq, Debug)]
struct AddressPartition {
    tag: u64,
    set: u64,
    block: u64,
}

#[derive(Debug)]
pub struct Statistics {
    hit: u32,
    miss: u32,
    eviction: u32,
}

impl Cache {
    /// Construct an empty, cold cache
    pub fn new(set_bits: u8, num_lines: u8, block_bits: u8) -> Cache {
        let total_sets = 2_u8.pow(set_bits as u32);
        let total_bytes = 2_u8.pow(block_bits as u32);

        let mut sets: Vec<Set> = Vec::with_capacity(total_sets as usize);

        for _ in 0..total_sets {
            let mut cache_lines = Vec::with_capacity(num_lines as usize);
            for _ in 0..num_lines {
                let mut bytes: Vec<u8> = Vec::with_capacity(total_bytes as usize);
                bytes.resize(total_bytes as usize, 0);

                cache_lines.push(Line {
                    valid: false,
                    tag: 0,
                    block: bytes.into_boxed_slice(),
                    access_time: Instant::now(),
                })
            }
            sets.push(Set { lines: cache_lines.into_boxed_slice() }); 
        }

        Cache { 
            sets: sets.into_boxed_slice(),
            set_bits: set_bits,
            block_bits: block_bits,
            tag_bits: 64_u8 - (set_bits + block_bits),
            num_lines: num_lines,
            stats: Statistics { hit: 0, miss: 0, eviction: 0 },
        }
    }

    /// Iterate over the memory access stream and simulate cache accesses
    pub fn operate_cache(&mut self, traces: Vec<MemoryAccess>) {
        for trace in traces {
            let parts = self.decompose(trace.address);
            
            match self.attempt_cache_hit(&parts) {
                true => continue,
                false => {},
            }
           
            match self.attempt_cache_store(&parts) {
                true => continue,
                false => {},
            }
            
            self.evict_cache_block(&parts);
        }
    }

    fn attempt_cache_hit(&mut self, parts: &AddressPartition) -> bool {
        for line in self.sets[parts.set as usize].lines.iter_mut() {
            if line.valid == true && line.tag == parts.tag {
                self.stats.hit += 1;
                line.access_time = Instant::now();
                return true;
            } else {
                self.stats.miss += 1;
                return false;
            }
        }
        false
    }

    fn attempt_cache_store(&mut self, parts: &AddressPartition) -> bool {
        for line in self.sets[parts.set as usize].lines.iter_mut() {
            if line.valid == false {
                line.valid = true;
                line.tag = parts.tag;
                return true;
            }
        }
        false
    }

    fn evict_cache_block(&mut self, parts: &AddressPartition) {
        let mut initial_time = self.sets[parts.set as usize].lines[0].access_time.clone();
        let mut id = 0;
        
        for (pos, line) in self.sets[parts.set as usize].lines.iter_mut().enumerate().skip(1) {
            if line.access_time < initial_time {
                initial_time = line.access_time;
                id = pos;
            }
        }

        self.sets[parts.set as usize].lines[id].valid = true;
        self.sets[parts.set as usize].lines[id].tag = parts.tag;
        self.sets[parts.set as usize].lines[id].access_time = Instant::now(); 
        self.stats.eviction += 1;
    }

    /// Decompose a 64-bit memory address into its constituent tag, set, and block bits
    fn decompose(&self, address: u64) -> AddressPartition {
        Cache::place_block(address, self.set_bits, self.block_bits)
    }

    fn place_block(address: u64, set_bits: u8, block_bits: u8) -> AddressPartition {
        let tag_bits = 64 - (set_bits + block_bits);
        AddressPartition {
            tag: address >> (set_bits + block_bits),
            set: (address << tag_bits) >> (tag_bits + block_bits),
            block: (address << (tag_bits + set_bits)) >> (tag_bits + set_bits),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_decomposition() {
        // (full address, set bits, block bits)
        let addresses: [(u64, u8, u8); 2] = [
            (0xFFFF_FFFF_FF_DEF_ABC, 12, 12),
            (0x12A8_FFFF_DEAD_CFFE, 16, 16),
        ];
        
        // (tag, set, block)
        let parts: [(u64, u64, u64); 2] = [
            (0xFFFF_FFFF_FF, 0xDEF, 0xABC),
            (0x12A8_FFFF, 0xDEAD, 0xCFFE),
        ];
        
        for i in 0..addresses.len() {
            assert_eq!(
                Cache::place_block(addresses[i].0, addresses[i].1, addresses[i].2), 
                AddressPartition {
                    tag: parts[i].0,
                    set: parts[i].1,
                    block: parts[i].2,
                }
            );
        }
    }
}
