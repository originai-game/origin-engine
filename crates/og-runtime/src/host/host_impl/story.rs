//! The story's bindings (StoryApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_story {
    ($host:ty) => {
        impl $crate::host_impl::api::StoryApi for $host {
            fn og_story_set_saliency(&self, world: u32, strategy: String) -> String {
                $crate::host_impl::answer($crate::og_story_set_saliency(world, strategy))
            }
            fn og_story_bark(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_story_bark(world, desc))
            }
            fn og_story_ambient(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_story_ambient(world, desc))
            }
            fn og_story_barks(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_story_barks(world))
            }
            fn og_story_load(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_story_load(world, path))
            }
            fn og_story_parse(&self, world: u32, text: String) -> String {
                $crate::host_impl::answer($crate::og_story_parse(world, text))
            }
            fn og_story_start(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_story_start(world, desc))
            }
            fn og_story_view(&self, world: u32, conversation: u32) -> String {
                $crate::host_impl::json($crate::og_story_view(world, conversation))
            }
            fn og_story_history(&self, world: u32, conversation: u32) -> String {
                $crate::host_impl::json($crate::og_story_history(world, conversation))
            }
            fn og_story_conversations(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_story_conversations(world))
            }
            fn og_story_advance(&self, world: u32, conversation: u32) -> String {
                $crate::host_impl::answer($crate::og_story_advance(world, conversation))
            }
            fn og_story_choose(&self, world: u32, conversation: u32, index: u32) -> String {
                $crate::host_impl::answer($crate::og_story_choose(world, conversation, index))
            }
            fn og_story_stop(&self, world: u32, conversation: u32) -> bool {
                $crate::og_story_stop(world, conversation)
            }
            fn og_story_get(&self, world: u32, name: String) -> String {
                $crate::host_impl::json($crate::og_story_get(world, name))
            }
            fn og_story_set(&self, world: u32, name: String, value: String) -> String {
                $crate::host_impl::answer($crate::og_story_set(world, name, value))
            }
            fn og_story_reset(&self, world: u32) {
                $crate::og_story_reset(world)
            }
            fn og_story_start_quest(&self, world: u32, id: String) -> String {
                $crate::host_impl::answer($crate::og_story_start_quest(world, id))
            }
            fn og_story_progress(
                &self,
                world: u32,
                quest: String,
                objective: String,
                amount: Option<u32>,
            ) -> bool {
                $crate::og_story_progress(world, quest, objective, amount)
            }
            fn og_story_set_stage(&self, world: u32, quest: String, stage: String) -> String {
                $crate::host_impl::answer($crate::og_story_set_stage(world, quest, stage))
            }
            fn og_story_complete_quest(&self, world: u32, quest: String) -> String {
                $crate::host_impl::answer($crate::og_story_complete_quest(world, quest))
            }
            fn og_story_fail_quest(&self, world: u32, quest: String) -> String {
                $crate::host_impl::answer($crate::og_story_fail_quest(world, quest))
            }
            fn og_story_quest(&self, world: u32, id: String) -> String {
                $crate::host_impl::json($crate::og_story_quest(world, id))
            }
            fn og_story_quests(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_story_quests(world))
            }
            fn og_story_track(&self, world: u32, id: String, on: bool) -> bool {
                $crate::og_story_track(world, id, on)
            }
            fn og_story_mark_seen(&self, world: u32, id: String) -> bool {
                $crate::og_story_mark_seen(world, id)
            }
            fn og_story_mark(&self, world: u32, id: String, desc: Option<String>) -> bool {
                $crate::og_story_mark(world, id, desc)
            }
        }
    };
}
