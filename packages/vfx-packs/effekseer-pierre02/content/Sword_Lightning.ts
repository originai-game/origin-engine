import { file } from '@origingame/engine/author';
// Converted from Effekseer by og vfx import.
// effekseer: node 0.1.1.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.1.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.1.2 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.1.3 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.1.2.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.2.0.0.0 "Tracker": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.2.0.0.1 "Particle": location over an f-curve: straightened
// effekseer: node 0.1.2.0.0.1.0 "Light": location over an f-curve: straightened
// effekseer: node 0.1.2.0.0.2.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.2.0.0.2.0 "SparkTracker": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.2.0.0.2.0.0 "SparkLight": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.1.2.0.0.2.0.0 "SparkLight": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.1.2.0.0.3 "Ray": location over an f-curve: straightened
// effekseer: node 0.1.2.0.0.3.0 "Light": location over an f-curve: straightened
// effekseer: node 0.2 "Tracker_1": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 0.2.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.5.0.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.5.0.0 "SparkTracker": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.2.0.5.0.0.0 "SparkLight": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.6 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.6 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.2.0.6 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.7 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.7 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.2.0.7 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.8 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.8 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.2.0.8 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.9 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.9 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.2.0.9 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.2.0.10.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.2.0.10.0.0.0.0 "Tracker": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.3 "Tracker_2": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 0.3.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.5.0.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.5.0.0 "SparkTracker": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.3.0.5.0.0.0 "SparkLight": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.6 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.6 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.3.0.6 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.7 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.7 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.3.0.7 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.8 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.8 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.3.0.8 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.9 "EmberVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.9 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.3.0.9 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.3.0.10.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.3.0.10.0.0.0.0 "Tracker": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4 "Tracker_3": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 0.4.0.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.0 "Light": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.1 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.1 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.2 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.2 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.3 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.3 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.4 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.4 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.5.0.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.5.0.0 "SparkTracker": nested generation: 52 instances above its generator folded into one emitter
// effekseer: node 0.4.0.5.0.0.0 "SparkLight": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.4.0.5.0 "SparkEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.6 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0 "FireEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.6 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.7 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.7 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.8 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.8 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.9 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.4.0.9 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.4.0.10.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.4.0.10.0.0.0.0 "Tracker": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.5 "Tracker_4": a ribbon or track's sideways motion: only its rise and turn are kept
// effekseer: node 0.5.0.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.0 "Light": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.1 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.1 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.1 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.2 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.2 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.2 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.3 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.3 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.3 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.4 "FireVariation": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.4 "FireVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.4 "FireVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.5.0.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.5.0.0 "SparkTracker": nested generation: 52 instances above its generator folded into one emitter
// effekseer: node 0.5.0.5.0.0.0 "SparkLight": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.5.0.5.0 "SparkEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.6 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0 "FireEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.6 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.7 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.7 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.8 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.8 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.9 "EmberVariation": nested generation: 13 instances above its generator folded into one emitter
// effekseer: node 0.5.0.9 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.5.0.10.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.5.0.10.0.0.0.0 "Tracker": nested generation: 26 instances above its generator folded into one emitter
// effekseer: node 0.6.1.0 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.6.1.1 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.6.1.2 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.6.1.3 "EmberVariation": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.6.2.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.6.2.0.0.0 "Tracker": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.6.2.0.0.1 "Particle": location over an f-curve: straightened
// effekseer: node 0.6.2.0.0.1.0 "Light": location over an f-curve: straightened
// effekseer: node 0.6.2.0.0.2.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.6.2.0.0.2.0 "SparkTracker": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.6.2.0.0.2.0.0 "SparkLight": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.6.2.0.0.2.0.0 "SparkLight": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.6.2.0.0.3 "Light": location over an f-curve: straightened
// effekseer: node 0.6.2.0.0.3.0 "Ray": location over an f-curve: straightened
// effekseer: node 0.7.0.0.0.2.0.0.4 "CloudLighting_Variation1": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.0.0.2.0.0.4 "CloudLighting_Variation1": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.0.2.0.0.5 "CloudLighting_Variation2": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.0.0.2.0.0.5 "CloudLighting_Variation2": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.0.2.0.0.6 "CloudLighting_Variation3": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.0.0.2.0.0.6 "CloudLighting_Variation3": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.0.2.0.0.7 "CloudLighting_Variation4": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.0.0.2.0.0.7 "CloudLighting_Variation4": nested generation: 12 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.1.0.0.0.0 "Tracker": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.1.0.0.0.0.0 "Light": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.2.0.0.0.0 "Tracker": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.2.0.0.0.0.0 "Light": nested generation: 30 instances above its generator folded into one emitter
// effekseer: node 0.7.0.0.3.0 "Wind": ring 18 instances: the first 4 drawn
// effekseer: node 0.7.0.1.0.0.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.0.0.0.0 "Tracker": nested generation: 88 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0 "String": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.0.0.0.1 "Light": location over an f-curve: straightened
// effekseer: node 0.7.0.1.0.0.0.0.0.1 "Light": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.7.0.1.0.0.0.0.0.2 "Particle": location over an f-curve: straightened
// effekseer: node 0.7.0.1.0.0.0.0.0.2 "Particle": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.7.0.1.0.0.0.0.0.2.0 "Light": location over an f-curve: straightened
// effekseer: node 0.7.0.1.0.0.0.0.0.2.0 "Light": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.7.0.1.0.0.0.0.0.3.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.0.0.0.3.0 "SparkTracker": nested generation: 88 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.0.0.0.3.0.0 "SparkLight": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.0.0.0.3.0.0 "SparkLight": nested generation: 88 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.0.0.0.4.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.0.0.0.4.0.0.0.0 "Tracker": nested generation: 88 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.0.0.0.4.0.0.0.0.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.0.0.0.4.0.0.0.0.0 "Light": nested generation: 88 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.0.0.0.5 "Light": location over an f-curve: straightened
// effekseer: node 0.7.0.1.0.0.0.0.0.5 "Light": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.7.0.1.0.0.0.0.0.5.0 "Ray": location over an f-curve: straightened
// effekseer: node 0.7.0.1.0.0.0.0.0.5.0 "Ray": a child of an instanced parent: its motion drawn up the world's y
// effekseer: node 0.7.0.1.0.0.1.0 "Debris_Variation1": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.0 "Debris_Variation1": nested generation: 33 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.0 "Debris_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.1 "Debris_Variation2": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.1 "Debris_Variation2": nested generation: 33 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.1 "Debris_Variation2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.2 "Debris_Variation3": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.3 "Debris_Variation4": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.5 "Burst1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.6 "Burst2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.7.0 "Smoke_Variation1": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1 "Finish": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.7.1 "Smoke_Variation2": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.7.2 "Smoke_Variation3": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.7.3 "Smoke_Variation4": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.8.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.8.0 "SparkTracker": nested generation: 99 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.8.0.0 "SparkLight": nested generation: 33 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.8 "SparkEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.9 "Fire_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.10 "Fire_Variation2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.11 "Fire_Variation3": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.12 "Fire_Variation4": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.0.1.0.0.1.13.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.13.0.0.0.0 "Tracker": nested generation: 594 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.13.0.0.0.0.0 "Light": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.0.1.0.0.1.13.0.0.0.0.0 "Light": nested generation: 594 instances above its generator folded into one emitter
// effekseer: node 0.7.0.1.0.0.1.14 "Blur": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.1.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.1.0.0.0 "Tracker": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.1.0.0.1 "Light": location over an f-curve: straightened
// effekseer: node 0.7.1.0.0.2 "Particle": location over an f-curve: straightened
// effekseer: node 0.7.1.0.0.2.0 "Light": location over an f-curve: straightened
// effekseer: node 0.7.1.0.0.3.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.1.0.0.3.0 "SparkTracker": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.1.0.0.3.0.0 "SparkLight": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.1.0.0.3.0.0 "SparkLight": nested generation: 11 instances above its generator folded into one emitter
// effekseer: node 0.7.1.0.0.4 "Light": location over an f-curve: straightened
// effekseer: node 0.7.1.0.0.4.0 "Ray": location over an f-curve: straightened
// effekseer: node 0.7.2.1 "Burst1": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.1 "Burst1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.2.2 "Burst2": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.2 "Burst2": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.2.3 "Burst3": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.3 "Burst3": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.2.4 "Fire_Variation1": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.4 "Fire_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.2.5 "Fire_Variation1": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.5 "Fire_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.2.6 "Fire_Variation1": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.6 "Fire_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.2.7 "Fire_Variation1": nested generation: 3 instances above its generator folded into one emitter
// effekseer: node 0.7.2.7 "Fire_Variation1": a fixed sprite turned at random: its mean plane drawn
// effekseer: node 0.7.3.0.0.0.0 "Tracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.3.0.0.0.0 "Tracker": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.7.3.0.0.0.1 "Particle": location over an f-curve: straightened
// effekseer: node 0.7.3.0.0.0.1.0 "Light": location over an f-curve: straightened
// effekseer: node 0.7.3.0.0.0.2.0 "SparkTracker": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.3.0.0.0.2.0 "SparkTracker": nested generation: 24 instances above its generator folded into one emitter
// effekseer: node 0.7.3.0.0.0.2.0.0 "SparkLight": nested generation: 8 instances above its generator folded into one emitter
// effekseer: node 0.7.3.0.0.0.2 "SparkEmitter": a moving ancestor: its descendants are placed where it starts
// effekseer: node 0.7.3.0.0.0.3 "Light": location over an f-curve: straightened
// effekseer: node 0.7.3.0.0.0.3.0 "Ray": location over an f-curve: straightened
// effekseer: more than 64 elements: the last 98 left out
export default file.vfx({
  "elements": [
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/model-LongSword_Blade"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-0-0.effect"
        },
        "scale": 0.12,
        "color": [
          0.029,
          0.0385,
          0.0478,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.6,
        1.2
      ],
      "rotate": [
        -62.3613,
        8.4356,
        15.65
      ],
      "life": 18.6667
    },
    {
      "node": {
        "kind": "mesh",
        "shape": {
          "kind": "model",
          "asset": {
            "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/model-LongSword_Handle"
          }
        },
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-0-1.effect"
        },
        "scale": 0.12,
        "color": [
          0.1905,
          0.187,
          0.1768,
          1.0
        ],
        "blend": "alpha"
      },
      "at": 0.0,
      "offset": [
        0.0,
        3.6,
        1.2
      ],
      "rotate": [
        -62.3613,
        8.4356,
        15.65
      ],
      "life": 18.6667
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-1-0.emitter"
        }
      },
      "at": 0.75,
      "offset": [
        0.5281,
        6.7547,
        2.852
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-1-1.emitter"
        }
      },
      "at": 1.0,
      "offset": [
        0.5281,
        6.7547,
        2.852
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-1-2.emitter"
        }
      },
      "at": 0.75,
      "offset": [
        0.5281,
        6.7547,
        2.852
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-1-3.emitter"
        }
      },
      "at": 1.0,
      "offset": [
        0.5281,
        6.7547,
        2.852
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-0.effect"
        },
        "width": [
          1.0,
          0.25
        ],
        "lifetime": 0.3,
        "color": [
          0.1898,
          0.1844,
          0.6859,
          0.7843
        ]
      },
      "at": 0.0,
      "offset": [
        0.0467,
        3.5157,
        1.3461
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "velocity": [
        3.3007,
        19.7169,
        10.3247
      ],
      "life": 0.3833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-1.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0257,
        3.7534,
        1.2803
      ],
      "rotate": [
        -61.121,
        8.777,
        15.4643
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-1-0.emitter"
        }
      },
      "at": 0.0,
      "offset": [
        0.0257,
        3.7534,
        1.2803
      ],
      "rotate": [
        -61.121,
        8.777,
        15.4643
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-2-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.05,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 0.3,
      "offset": [
        1.0369,
        9.4308,
        4.4435
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "velocity": [
        3.3007,
        19.7169,
        10.3247
      ],
      "life": 0.9833
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-2-0-0.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        1.0369,
        9.4308,
        4.4435
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ],
      "velocity": [
        3.3007,
        19.7169,
        10.3247
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-3.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        0.0257,
        3.7534,
        1.2803
      ],
      "rotate": [
        -61.121,
        8.777,
        15.4643
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-1-2-0-0-3-0.emitter"
        }
      },
      "at": 0.3,
      "offset": [
        0.0257,
        3.7534,
        1.2803
      ],
      "rotate": [
        -61.121,
        8.777,
        15.4643
      ],
      "until": 6.3333
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2.effect"
        },
        "width": [
          8.0,
          2.0
        ],
        "lifetime": 0.2167,
        "color": [
          0.0478,
          0.2195,
          1.0,
          1.0
        ]
      },
      "at": 2.1167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        -62.3613,
        8.4356,
        15.65
      ],
      "life": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-0.emitter"
        }
      },
      "at": 2.1167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-1.emitter"
        }
      },
      "at": 2.1167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-2.emitter"
        }
      },
      "at": 2.2167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-3.emitter"
        }
      },
      "at": 2.3167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-4.emitter"
        }
      },
      "at": 2.4167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-5-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.05,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 2.1542,
      "offset": [
        -3.6913,
        8.7782,
        5.3687
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ],
      "velocity": [
        -14.8297,
        30.5911,
        22.3969
      ],
      "life": 0.8167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-5-0-0-0.parent"
        }
      },
      "at": 2.1167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-6.emitter"
        }
      },
      "at": 2.1167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-7.emitter"
        }
      },
      "at": 2.1167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-8.emitter"
        }
      },
      "at": 2.1167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-9.emitter"
        }
      },
      "at": 2.1167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-10-0-0-0-0.effect"
        },
        "width": [
          4.0,
          1.0
        ],
        "lifetime": 0.0667,
        "color": [
          0.7586,
          0.7083,
          0.937,
          0.9708
        ],
        "orbit": [
          0.2,
          10.0
        ]
      },
      "at": 2.4167,
      "offset": [
        -3.0015,
        22.5774,
        13.1126
      ],
      "rotate": [
        29.992,
        15.4764,
        50.7786
      ],
      "velocity": [
        -304.413,
        -333.6826,
        -95.2724
      ],
      "life": 0.085
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3.effect"
        },
        "width": [
          8.0,
          2.0
        ],
        "lifetime": 0.2167,
        "color": [
          0.0478,
          0.2195,
          1.0,
          1.0
        ]
      },
      "at": 3.0,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        -62.3613,
        8.4356,
        15.65
      ],
      "life": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-0.emitter"
        }
      },
      "at": 3.0,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-1.emitter"
        }
      },
      "at": 3.0,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-2.emitter"
        }
      },
      "at": 3.1,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-3.emitter"
        }
      },
      "at": 3.2,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-4.emitter"
        }
      },
      "at": 3.3,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-5-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.05,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 3.0375,
      "offset": [
        -3.6913,
        8.7782,
        5.3687
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ],
      "velocity": [
        -14.8297,
        30.5911,
        22.3969
      ],
      "life": 0.8167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-5-0-0-0.parent"
        }
      },
      "at": 3.0,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        29.992,
        15.4764,
        -8.7554
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-6.emitter"
        }
      },
      "at": 3.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-7.emitter"
        }
      },
      "at": 3.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-8.emitter"
        }
      },
      "at": 3.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-9.emitter"
        }
      },
      "at": 3.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-3-0-10-0-0-0-0.effect"
        },
        "width": [
          4.0,
          1.0
        ],
        "lifetime": 0.0667,
        "color": [
          0.7586,
          0.7083,
          0.937,
          0.9708
        ],
        "orbit": [
          0.2,
          10.0
        ]
      },
      "at": 3.3,
      "offset": [
        -3.0015,
        22.5774,
        13.1126
      ],
      "rotate": [
        29.992,
        15.4764,
        50.7786
      ],
      "velocity": [
        -304.413,
        -333.6826,
        -95.2724
      ],
      "life": 0.085
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4.effect"
        },
        "width": [
          8.0,
          2.0
        ],
        "lifetime": 0.2167,
        "color": [
          0.0478,
          0.2195,
          1.0,
          1.0
        ]
      },
      "at": 4.0,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        -62.3613,
        8.4356,
        15.65
      ],
      "life": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-0.emitter"
        }
      },
      "at": 4.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-1.emitter"
        }
      },
      "at": 4.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-2.emitter"
        }
      },
      "at": 4.1,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-3.emitter"
        }
      },
      "at": 4.2,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-4.emitter"
        }
      },
      "at": 4.3,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-5-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.05,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 4.0375,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        141.2446
      ],
      "life": 0.8167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-5-0-0-0.parent"
        }
      },
      "at": 4.0375,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        -72.0983,
        37.1065,
        -160.4523
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-6.parent"
        }
      },
      "at": 4.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-7.parent"
        }
      },
      "at": 4.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-8.parent"
        }
      },
      "at": 4.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-9.parent"
        }
      },
      "at": 4.0,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-4-0-10-0-0-0-0.effect"
        },
        "width": [
          4.0,
          1.0
        ],
        "lifetime": 0.0667,
        "color": [
          0.7586,
          0.7083,
          0.937,
          0.9708
        ],
        "orbit": [
          0.2,
          10.0
        ]
      },
      "at": 4.3,
      "offset": [
        1.4474,
        13.4001,
        6.3936
      ],
      "rotate": [
        29.992,
        15.4764,
        50.7786
      ],
      "velocity": [
        -289.5833,
        -364.2737,
        -117.6693
      ],
      "life": 0.085
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5.effect"
        },
        "width": [
          8.0,
          2.0
        ],
        "lifetime": 0.2167,
        "color": [
          0.0478,
          0.2195,
          1.0,
          1.0
        ]
      },
      "at": 4.5167,
      "offset": [
        1.0269,
        9.7341,
        4.4121
      ],
      "rotate": [
        -62.3613,
        8.4356,
        15.65
      ],
      "life": 0.2
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-0.emitter"
        }
      },
      "at": 4.5167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-1.emitter"
        }
      },
      "at": 4.5167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-2.emitter"
        }
      },
      "at": 4.6167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-3.emitter"
        }
      },
      "at": 4.7167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-4.emitter"
        }
      },
      "at": 4.8167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "trail",
        "effect": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-5-0-0.effect"
        },
        "width": [
          0.0,
          0.0
        ],
        "lifetime": 0.05,
        "color": [
          1.0,
          1.0,
          1.0,
          1.0
        ]
      },
      "at": 4.5542,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        0.0,
        -72.2715,
        61.1999
      ],
      "life": 0.8167
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-5-0-0-0.parent"
        }
      },
      "at": 4.5542,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        81.2446
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-6.parent"
        }
      },
      "at": 4.5167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-7.parent"
        }
      },
      "at": 4.5167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-8.parent"
        }
      },
      "at": 4.5167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    },
    {
      "node": {
        "kind": "emitter",
        "asset": {
          "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-9.parent"
        }
      },
      "at": 4.5167,
      "offset": [
        -3.1352,
        7.631,
        4.5289
      ],
      "rotate": [
        29.992,
        15.4764,
        22.2083
      ]
    }
  ],
  "duration": 17.3333,
  "looping": true
});
