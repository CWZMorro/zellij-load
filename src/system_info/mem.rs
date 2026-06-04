use sysinfo::System;

#[derive(Default)]
pub struct MemUsage {
    pub total: u64,
    pub idle: u64,
}

impl MemUsage {
    pub fn update(&mut self, sys: &mut System) {
        sys.refresh_memory();
        self.total = sys.total_memory();
        self.idle = sys.available_memory();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_update_produces_valid_values() {
        let mut sys = System::new_all();
        let mut usage = MemUsage::default();
        usage.update(&mut sys);
        assert!(usage.total > 0, "total memory should be nonzero");
        assert!(usage.idle <= usage.total, "idle cannot exceed total");
    }
}
