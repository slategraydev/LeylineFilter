use crate::core::traits::{AudioModule, EngineState, ModuleConfig, ModuleInfo};

/// Manages the sequence of audio modules and their processing order.
pub struct SignalChain {
    modules: Vec<Box<dyn AudioModule>>,
    sample_rate: f32,
}

impl SignalChain {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            modules: Vec::with_capacity(32),
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for module in self.modules.iter_mut() {
            module.prepare(sample_rate);
        }
    }

    pub fn add_module(&mut self, module: Box<dyn AudioModule>) -> String {
        let id = module.id().to_string();
        self.modules.push(module);
        id
    }

    pub fn remove_module(&mut self, id: &str) -> Option<Box<dyn AudioModule>> {
        if let Some(pos) = self.modules.iter().position(|m| m.id() == id) {
            Some(self.modules.remove(pos))
        } else {
            None
        }
    }

    pub fn reorder(&mut self, order: &[String]) {
        let mut write_head = 0;
        for target_id in order {
            // Find target_id in modules[write_head..]
            let mut found_index = None;
            for i in write_head..self.modules.len() {
                if self.modules[i].id() == target_id {
                    found_index = Some(i);
                    break;
                }
            }

            if let Some(idx) = found_index {
                self.modules.swap(write_head, idx);
                write_head += 1;
            }
        }
    }

    pub fn process(&mut self, audio: &mut [f32]) {
        for module in self.modules.iter_mut() {
            module.process(audio);
        }
    }

    pub fn update_config(&mut self, config: &ModuleConfig) {
        // Broad update: many modules might want to know about a config change
        // Or we could target specific modules if we had a better mapping
        for module in self.modules.iter_mut() {
            module.update_config(config);
        }
    }

    pub fn update_module_param(&mut self, id: &str, _param: &str, _value: f32) {
        // This might require adding a `set_param` method to the `AudioModule` trait
        // For now, we'll skip this or implement it if we update the trait.
        log::warn!("set_param not yet implemented for module {}", id);
    }

    pub fn modules(&self) -> &Vec<Box<dyn AudioModule>> {
        &self.modules
    }

    pub fn latency_samples(&self) -> usize {
        self.modules.iter().map(|m| m.latency_samples()).sum()
    }

    pub fn get_state(&self, is_running: bool) -> EngineState {
        let modules = self
            .modules
            .iter()
            .map(|m| ModuleInfo {
                id: m.id().to_string(),
                name: m.name().to_string(),
                category: m.category(),
                enabled: m.is_enabled(),
                config: m.get_config(),
            })
            .collect();

        EngineState {
            modules,
            is_running,
            sample_rate: self.sample_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::modules::ModuleFactory;

    #[test]
    fn test_chain_management() {
        let mut chain = SignalChain::new(48000.0);

        // Add modules
        let m1 = ModuleFactory::create("Gain", 48000.0).unwrap();
        let id1 = chain.add_module(m1);

        let m2 = ModuleFactory::create("Filter", 48000.0).unwrap();
        let id2 = chain.add_module(m2);

        assert_eq!(chain.modules().len(), 2);

        // Reorder
        chain.reorder(&[id2.clone(), id1.clone()]);
        assert_eq!(chain.modules()[0].id(), id2);
        assert_eq!(chain.modules()[1].id(), id1);

        // State
        let state = chain.get_state(true);
        assert_eq!(state.modules.len(), 2);
        assert!(state.is_running);

        // Remove
        let removed = chain.remove_module(&id1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id(), id1);

        assert_eq!(chain.modules().len(), 1);
        assert_eq!(chain.modules()[0].id(), id2);
    }
}
