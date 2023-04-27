use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationID(u32);

#[derive(Debug)]
pub struct AnimationSystem {
    animations: HashMap<AnimationID, AnimationData>,
    next_id: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct AnimationData {
    pub from: usize,
    pub to: usize,
    pub is_playing: bool,
    pub current_frame: usize,
    pub playback_pos_ms: u32,
    pub frame_times_ms: Vec<u32>,
    pub total_length_ms: u32,
}

impl AnimationSystem {
    pub fn new() -> Self {
        AnimationSystem {
            animations: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_animation(
        &mut self,
        from: usize,
        to: usize,
        frame_periods_ms: &[u32],
    ) -> AnimationID {
        let id = AnimationID(self.next_id);
        self.next_id += 1;

        let mut total_length_ms = 0;
        let mut frame_times_ms = Vec::new();
        for period in frame_periods_ms {
            let start_time = total_length_ms;
            frame_times_ms.push(start_time);
            total_length_ms += period;
        }

        self.animations.insert(
            id,
            AnimationData {
                from,
                to,
                is_playing: false,
                current_frame: from,
                playback_pos_ms: 0,
                frame_times_ms,
                total_length_ms,
            },
        );

        id
    }

    pub fn reload_animation(
        &mut self,
        id: AnimationID,
        from: usize,
        to: usize,
        frame_periods_ms: &[u32],
    ) {
        let mut total_length_ms = 0;
        let mut frame_times_ms = Vec::new();
        for period in frame_periods_ms {
            let start_time = total_length_ms;
            frame_times_ms.push(start_time);
            total_length_ms += period;
        }

        self.animations.insert(
            id,
            AnimationData {
                from,
                to,
                is_playing: false,
                current_frame: from,
                playback_pos_ms: 0,
                frame_times_ms,
                total_length_ms,
            },
        );
    }

    #[allow(dead_code)]
    pub fn start_animation(&mut self, animation_id: AnimationID) {
        self.animations.get_mut(&animation_id).unwrap().is_playing = true;
    }

    #[allow(dead_code)]
    pub fn stop_animation(&mut self, animation_id: AnimationID) {
        self.animations.get_mut(&animation_id).unwrap().is_playing = false;
    }

    #[allow(dead_code)]
    pub fn restart_animation(&mut self, animation_id: AnimationID) {
        self.reset_animation(animation_id);
        self.start_animation(animation_id);
    }

    #[allow(dead_code)]
    pub fn reset_animation(&mut self, animation_id: AnimationID) {
        let mut animation = self.animations.get_mut(&animation_id).unwrap();
        animation.current_frame = animation.from;
        animation.playback_pos_ms = 0;
    }

    #[allow(dead_code)]
    pub fn step_to_next_frame(&mut self, animation_id: AnimationID) {
        let animation = self.animations.get_mut(&animation_id).unwrap();
        animation.current_frame = if animation.current_frame + 1 > animation.to {
            animation.from
        } else {
            animation.current_frame + 1
        }
    }

    pub fn update(&mut self, delta_time_ms: u32) {
        for (_, animation) in &mut self.animations {
            if !animation.is_playing {
                continue;
            }

            let next_pos = (animation.playback_pos_ms + delta_time_ms) % animation.total_length_ms;
            animation.playback_pos_ms = next_pos;

            for (i, frame_time) in animation.frame_times_ms.iter().enumerate().rev() {
                if animation.playback_pos_ms >= *frame_time {
                    animation.current_frame = animation.from + i;
                    break;
                }
            }
        }
    }

    pub fn current_frame(&mut self, animation_id: AnimationID) -> usize {
        self.animations[&animation_id].current_frame
    }
}

pub fn add_asperite_sprite_sheet_animation(
    animation_system: &mut AnimationSystem,
    sprite_sheet: &aseprite::SpritesheetData,
    frame_tag_name: &str,
) -> AnimationID {
    let frame_tag = sprite_sheet_frame_tag(sprite_sheet, frame_tag_name);
    let from = frame_tag.from as usize;
    let to = frame_tag.to as usize;
    let frame_periods_ms = sprite_sheet.frames[from..=to]
        .iter()
        .map(|frame| frame.duration)
        .collect::<Vec<u32>>();

    animation_system.add_animation(from, to, &frame_periods_ms)
}

pub fn reload_aseperite_sprite_sheet_animation(
    id: AnimationID,
    animation_system: &mut AnimationSystem,
    sprite_sheet: &aseprite::SpritesheetData,
    frame_tag_name: &str,
) {
    let frame_tag = sprite_sheet_frame_tag(sprite_sheet, frame_tag_name);
    let from = frame_tag.from as usize;
    let to = frame_tag.to as usize;
    let frame_periods_ms = sprite_sheet.frames[from..=to]
        .iter()
        .map(|frame| frame.duration)
        .collect::<Vec<u32>>();

    animation_system.reload_animation(id, from, to, &frame_periods_ms);
}

fn sprite_sheet_frame_tag(
    sprite_sheet: &aseprite::SpritesheetData,
    frame_tag_name: &str,
) -> aseprite::Frametag {
    let frame_tags = sprite_sheet.meta.frame_tags.as_ref().unwrap();
    let frame_tag = frame_tags
        .iter()
        .find(|tag| tag.name == frame_tag_name)
        .map(|f| f.clone())
        .unwrap();

    frame_tag
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: panics if to - from doesn't match length of frames

    #[test]
    fn initially_returns_first_frame() {
        let mut animation_system = AnimationSystem::new();
        let frame_periods_ms = [0, 100];
        let (from, to) = (0, 1);
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 0);
    }

    #[test]
    fn when_frame_period_elapsed_then_next_frame_is_selected() {
        let mut animation_system = AnimationSystem::new();
        let frame_periods_ms = [100, 100];
        let (from, to) = (1, 2);
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        let delta_time_ms = 100;
        animation_system.start_animation(animation_id);
        animation_system.update(delta_time_ms);
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 2);
    }

    #[test]
    fn when_elapsed_time_exceeds_period_time_then_playback_wraps_around() {
        let mut animation_system = AnimationSystem::new();
        let (from, to) = (1, 4);
        let frame_periods_ms = [100, 100, 100, 100];
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        let delta_time_ms = 400;
        animation_system.start_animation(animation_id);
        animation_system.update(delta_time_ms);
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 1);
    }

    #[test]
    fn if_playback_stopped_when_frame_period_elapsed_then_same_frame_is_selected() {
        let mut animation_system = AnimationSystem::new();
        let (from, to) = (1, 2);
        let frame_periods_ms = [100, 100];
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        let delta_time_ms = 100;
        animation_system.start_animation(animation_id);
        animation_system.stop_animation(animation_id);
        animation_system.update(delta_time_ms);
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 1);
    }

    #[test]
    fn frames_can_be_manually_incremented() {
        let mut animation_system = AnimationSystem::new();
        let frame_periods_ms = [100, 100];
        let (from, to) = (1, 2);
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        animation_system.step_to_next_frame(animation_id);
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 2);
    }

    #[test]
    fn step_to_next_frame_wraps_around() {
        let mut animation_system = AnimationSystem::new();
        let frame_periods_ms = [100, 100, 100];
        let (from, to) = (0, 2);
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        animation_system.step_to_next_frame(animation_id); // -> 1
        animation_system.step_to_next_frame(animation_id); // -> 2
        animation_system.step_to_next_frame(animation_id); // -> 0
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 0);
    }

    #[test]
    fn reloading_animation_resets_current_frame_to_from_value() {
        let (from, to) = (3, 4);
        let frame_periods_ms = [100, 100];
        let mut animation_system = AnimationSystem::new();
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        animation_system.step_to_next_frame(animation_id);
        animation_system.reload_animation(animation_id, from, to, &frame_periods_ms);
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 3);
    }

    #[test]
    fn reloading_animation_can_use_new_frames() {
        let (from, to) = (0, 1);
        let frame_periods_ms = [100, 100];
        let mut animation_system = AnimationSystem::new();
        let animation_id = animation_system.add_animation(from, to, &frame_periods_ms);

        let (from, to) = (2, 4);
        let frame_periods_ms = [100, 100, 100];
        animation_system.reload_animation(animation_id, from, to, &frame_periods_ms);
        animation_system.step_to_next_frame(animation_id); // 2 -> 3
        let frame = animation_system.current_frame(animation_id);

        assert_eq!(frame, 3);
    }
}
