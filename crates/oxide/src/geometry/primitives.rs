use crate::model::ModelVertex;

pub const PENT_V: &[ModelVertex] = &[
        ModelVertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.00759614],
                normal: [0.0, 0.0, 1.0],
        }, // A
        ModelVertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.43041354],
                normal: [0.0, 0.0, 1.0],
        }, // B
        ModelVertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.949397],
                normal: [0.0, 0.0, 1.0],
        }, // C
        ModelVertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.84732914],
                normal: [0.0, 0.0, 1.0],
        }, // D
        ModelVertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.2652641],
                normal: [0.0, 0.0, 1.0],
        }, // E
];

pub const PENT_I: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub const SQ_V: &[ModelVertex] = &[
        ModelVertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        },
        ModelVertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        },
        ModelVertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        },
        ModelVertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        },
];

pub const SQ_I: &[u16] = &[0, 1, 2, 2, 3, 0];

pub const TRI_V: &[ModelVertex] = &[
        ModelVertex {
                position: [-0.0, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        }, // A
        ModelVertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        }, // B
        ModelVertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
        }, // C
];

pub const TRI_I: &[u16] = &[0, 1, 2];
