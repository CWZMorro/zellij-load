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
