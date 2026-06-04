use sysinfo::System;

#[derive(Default)]
pub struct CpuUsage {
    pub total: f32,
}

impl CpuUsage {
    pub fn sample(&mut self, sys: &mut System) {
        sys.refresh_cpu_all();
    }

    pub fn read(&mut self, sys: &mut System) {
        sys.refresh_cpu_all();
        self.total = sys.global_cpu_usage();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_read_produces_valid_percentage() {
        let mut sys = System::new_all();
        let mut usage = CpuUsage::default();
        usage.sample(&mut sys);
        usage.read(&mut sys);
        assert!(
            (0.0..=100.0).contains(&usage.total),
            "cpu usage {} is out of range",
            usage.total
        );
    }
}
