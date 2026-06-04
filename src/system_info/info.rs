use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SystemMessage {
    pub cpu_usage: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub gpu_info: Option<GPU>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GPU {
    pub name: String,
    pub memory_used: u64,
    pub memory_total: u64,
    pub gpu_utilization: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_message_roundtrip() {
        let msg = SystemMessage {
            cpu_usage: 42.5,
            mem_used: 8_000_000_000,
            mem_total: 16_000_000_000,
            gpu_info: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: SystemMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.cpu_usage, msg.cpu_usage);
        assert_eq!(decoded.mem_used, msg.mem_used);
        assert_eq!(decoded.mem_total, msg.mem_total);
        assert!(decoded.gpu_info.is_none());
    }

    #[test]
    fn system_message_with_gpu_roundtrip() {
        let msg = SystemMessage {
            cpu_usage: 10.0,
            mem_used: 4_000_000_000,
            mem_total: 16_000_000_000,
            gpu_info: Some(GPU {
                name: "NVIDIA RTX 4090".to_string(),
                memory_used: 2_000_000_000,
                memory_total: 24_000_000_000,
                gpu_utilization: 75,
            }),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: SystemMessage = serde_json::from_str(&json).unwrap();
        let gpu = decoded.gpu_info.unwrap();
        assert_eq!(gpu.name, "NVIDIA RTX 4090");
        assert_eq!(gpu.memory_used, 2_000_000_000);
        assert_eq!(gpu.memory_total, 24_000_000_000);
        assert_eq!(gpu.gpu_utilization, 75);
    }

    #[test]
    fn system_message_default_has_no_gpu() {
        let msg = SystemMessage::default();
        assert!(msg.gpu_info.is_none());
        assert_eq!(msg.cpu_usage, 0.0);
        assert_eq!(msg.mem_used, 0);
        assert_eq!(msg.mem_total, 0);
    }
}
