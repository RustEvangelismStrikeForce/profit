#[derive(Clone, Debug, PartialEq)]
pub struct Network<
    const INPUT_SIZE: usize,
    const OUTPUT_SIZE: usize,
    const HIDDEN_LAYER_SIZE: usize,
    const HIDDEN_LAYERS: usize,
> {
    input_layer: [f32; INPUT_SIZE],
    output_layer: [f32; OUTPUT_SIZE],
    hidden_layers: [[f32; HIDDEN_LAYER_SIZE]; HIDDEN_LAYERS],
    weights: Vec<f32>,
}

