//! Simulation-coordinate stamps for the transforms actually supplying the
//! velocity buffers. Captures never advance displayed history. Carried by
//! the same checkpoint as those transforms, not a wall-clock timer.

use og_render::Camera;

#[derive(Clone, Copy, Debug)]
struct Sample {
    tick: u64,
    seconds: f64,
    camera: Camera,
}

#[derive(Clone, Default)]
pub(crate) struct MotionClock {
    current: Option<Sample>,
    previous: Option<Sample>,
    drawn: Option<Sample>,
    pub seconds: f32,
}

impl MotionClock {
    pub fn shift(&mut self, delta: glam::DVec3) {
        for sample in [&mut self.current, &mut self.previous, &mut self.drawn]
            .into_iter()
            .flatten()
        {
            sample.camera.position = (sample.camera.position.as_dvec3() + delta).as_vec3();
            sample.camera.target = (sample.camera.target.as_dvec3() + delta).as_vec3();
        }
    }

    pub fn sample(
        &mut self,
        time: og_core::Time,
        alpha: Option<f32>,
        display: bool,
        from_drawn: bool,
        camera: Camera,
    ) -> Option<Camera> {
        let coordinate = time.tick as f64 - alpha.map_or(0.0, |a| 1.0 - f64::from(a));
        let sample = Sample {
            tick: time.tick,
            seconds: coordinate / f64::from(time.rate),
            camera,
        };
        if self.current.is_some_and(|c| c.tick != time.tick) {
            self.previous = self.current;
        }
        self.current = Some(sample);
        let before = if from_drawn {
            self.drawn
        } else {
            self.previous
        };
        self.seconds = before.map_or(0.0, |b| (sample.seconds - b.seconds).max(0.0) as f32);
        if display {
            self.drawn = Some(sample);
        }
        before.map(|b| b.camera)
    }
}
