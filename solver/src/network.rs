use rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct Network<
    const INPUT_SIZE: usize,
    const OUTPUT_SIZE: usize,
    const HIDDEN_LAYER_SIZE: usize,
    const HIDDEN_LAYERS: usize,
> {
    /// weights of the input layer
    input_layer: [f32; INPUT_SIZE],
    /// weights of the input layer
    first_hidden_layer: [[f32; INPUT_SIZE]; HIDDEN_LAYER_SIZE],
    /// weights of the hidden layers
    hidden_layers: [[[f32; HIDDEN_LAYER_SIZE]; HIDDEN_LAYER_SIZE]; HIDDEN_LAYERS],
    /// weights of the output layer
    output_layer: [[f32; HIDDEN_LAYER_SIZE]; OUTPUT_SIZE],
}

impl<
        const INPUT_SIZE: usize,
        const OUTPUT_SIZE: usize,
        const HIDDEN_LAYER_SIZE: usize,
        const HIDDEN_LAYERS: usize,
    > Network<INPUT_SIZE, OUTPUT_SIZE, HIDDEN_LAYER_SIZE, HIDDEN_LAYERS>
{
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            input_layer: rng.gen(),
            first_hidden_layer: rng.gen(),
            hidden_layers: rng.gen(),
            output_layer: rng.gen(),
        }
    }

    pub fn calc(&self, mut input: [f32; INPUT_SIZE]) -> [f32; OUTPUT_SIZE] {
        // input
        for i in 0..INPUT_SIZE {
            input[i] *= self.input_layer[i];
        }

        // hidden
        let mut hidden = [[0.0; HIDDEN_LAYER_SIZE]; 2];
        let mut current = 0;
        for i in 0..HIDDEN_LAYER_SIZE {
            for j in 0..INPUT_SIZE {
                hidden[current][i] += input[j] * self.first_hidden_layer[i][j];
            }
            hidden[current][i] = activation_funciton(hidden[current][i]);
        }
        current = (current + 1) % 2;

        for l in 0..HIDDEN_LAYERS {
            let other = (current + 1) % 2;
            for i in 0..HIDDEN_LAYER_SIZE {
                for j in 0..HIDDEN_LAYER_SIZE {
                    hidden[current][i] += hidden[other][j] * self.hidden_layers[l][i][j];
                }
                hidden[current][i] = activation_funciton(hidden[current][i]);
            }
            current = other;
        }

        // output
        let other = (current + 1) % 2;
        let mut output = [0.0; OUTPUT_SIZE];
        for i in 0..OUTPUT_SIZE {
            for j in 0..HIDDEN_LAYER_SIZE {
                output[i] += hidden[other][j] * self.output_layer[i][j];
            }
            output[i] = activation_funciton(output[i]);
        }

        output
    }
}

#[inline(always)]
pub fn activation_funciton(val: f32) -> f32 {
    val.tanh()
}
