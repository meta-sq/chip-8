use rand::random;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_DEPTH: usize = 16;
const KEY_COUNT: usize = 16;
pub const STARTING_ADDRESS: u16 = 0x200;
const FONT_DATA_SIZE: usize = 80;

const FONT_DATA: [u8; FONT_DATA_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    program_counter: u16,
    memory: [u8; MEMORY_SIZE],
    framebuffer: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    general_registers: [u8; REGISTER_COUNT],
    index_register: u16,
    stack_pointer: u16,
    call_stack: [u16; STACK_DEPTH],
    input_keys: [bool; KEY_COUNT],
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    /// Creates a new Chip-8 emulator instance
    pub fn initialize() -> Self {
        let mut instance = Self {
            program_counter: STARTING_ADDRESS,
            memory: [0; MEMORY_SIZE],
            framebuffer: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            general_registers: [0; REGISTER_COUNT],
            index_register: 0,
            stack_pointer: 0,
            call_stack: [0; STACK_DEPTH],
            input_keys: [false; KEY_COUNT],
            delay_timer: 0,
            sound_timer: 0,
        };

        instance.memory[..FONT_DATA_SIZE].copy_from_slice(&FONT_DATA);
        instance
    }

    /// Resets the emulator state
    pub fn reset(&mut self) {
        self.program_counter = STARTING_ADDRESS;
        self.memory = [0; MEMORY_SIZE];
        self.framebuffer = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        self.general_registers = [0; REGISTER_COUNT];
        self.index_register = 0;
        self.stack_pointer = 0;
        self.call_stack = [0; STACK_DEPTH];
        self.input_keys = [false; KEY_COUNT];
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.memory[..FONT_DATA_SIZE].copy_from_slice(&FONT_DATA);
    }

    /// Pushes a value onto the stack
    fn push_to_stack(&mut self, value: u16) {
        self.call_stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    /// Pops a value from the stack
    fn pop_from_stack(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.call_stack[self.stack_pointer as usize]
    }

    /// Executes one CPU cycle
    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);
    }

    /// Retrieves the display buffer
    pub fn get_framebuffer(&self) -> &[bool] {
        &self.framebuffer
    }

    /// Sets the state of a specific key
    pub fn set_key_state(&mut self, key_index: usize, is_pressed: bool) {
        self.input_keys[key_index] = is_pressed;
    }

    /// Loads a program into memory starting at the program start address
    pub fn load_program(&mut self, program_data: &[u8]) {
        let start_address = STARTING_ADDRESS as usize;
        let end_address = start_address + program_data.len();
        self.memory[start_address..end_address].copy_from_slice(program_data);
    }

    /// Fetches the next opcode from memory
    fn fetch_opcode(&mut self) -> u16 {
        let high_byte = self.memory[self.program_counter as usize] as u16;
        let low_byte = self.memory[(self.program_counter + 1) as usize] as u16;
        self.program_counter += 2;
        (high_byte << 8) | low_byte
    }

    /// Decodes and executes a given opcode
    fn execute_opcode(&mut self, opcode: u16) {
        let nibble1 = (opcode & 0xF000) >> 12;
        let nibble2 = (opcode & 0x0F00) >> 8;
        let nibble3 = (opcode & 0x00F0) >> 4;
        let nibble4 = opcode & 0x000F;

        match (nibble1, nibble2, nibble3, nibble4) {
            (1, _, _, _) => {
                // Jump to address NNN
                let address = opcode & 0x0FFF;
                self.program_counter = address;
            },
            (2, _, _, _) => {
                // Call subroutine at NNN
                let address = opcode & 0x0FFF;
                self.push_to_stack(self.program_counter);
                self.program_counter = address;
            },
            (3, _, _, _) => {
                // Skip next instruction if VX == NN
                let x = nibble2 as usize;
                let value = (opcode & 0x00FF) as u8;
                if self.general_registers[x] == value {
                    self.program_counter += 2;
                }
            },
            (4, _, _, _) => {
                // Skip next instruction if VX != NN
                let x = nibble2 as usize;
                let value = (opcode & 0x00FF) as u8;
                if self.general_registers[x] != value {
                    self.program_counter += 2;
                }
            },
            (5, _, _, 0) => {
                // Skip next instruction if VX == VY
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                if self.general_registers[x] == self.general_registers[y] {
                    self.program_counter += 2;
                }
            },
            (6, _, _, _) => {
                // Set VX to NN
                let x = nibble2 as usize;
                let value = (opcode & 0x00FF) as u8;
                self.general_registers[x] = value;
            },
            (7, _, _, _) => {
                // Add NN to VX
                let x = nibble2 as usize;
                let value = (opcode & 0x00FF) as u8;
                self.general_registers[x] = self.general_registers[x].wrapping_add(value);
            },
            (8, _, _, 0) => {
                // Set VX to VY
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.general_registers[x] = self.general_registers[y];
            },
            (0,0,0xE,0xE) => {let retaddr=self.pop_from_stack(); self.program_counter=retaddr;},//return to addr
            (0,0,0xE,0) => {self.framebuffer =[false;DISPLAY_WIDTH*DISPLAY_HEIGHT];}, //clear screen
            (0,0,0,0) => return,//nop
            (8, _, _, 1) => {
                // Set VX to VX OR VY
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.general_registers[x] |= self.general_registers[y];
            },
            (8, _, _, 2) => {
                // Set VX to VX AND VY
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.general_registers[x] &= self.general_registers[y];
            },
            (8, _, _, 3) => {
                // Set VX to VX XOR VY
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.general_registers[x] ^= self.general_registers[y];
            },
            (8, _, _, 4) => {
                // Add VY to VX, set VF to carry
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                let (sum, carry) = self.general_registers[x].overflowing_add(self.general_registers[y]);
                self.general_registers[x] = sum;
                self.general_registers[0xF] = if carry { 1 } else { 0 };
            },
            (8, _, _, 5) => {
                // Subtract VY from VX, set VF to NOT borrow
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                let (diff, borrow) = self.general_registers[x].overflowing_sub(self.general_registers[y]);
                self.general_registers[x] = diff;
                self.general_registers[0xF] = if borrow { 0 } else { 1 };
            },
            (8, _, _, 6) => {
                // Shift VX right by 1, set VF to LSB
                let x = nibble2 as usize;
                self.general_registers[0xF] = self.general_registers[x] & 0x1;
                self.general_registers[x] >>= 1;
            },
            (8, _, _, 7) => {
                // Set VX to VY - VX, set VF to NOT borrow
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                let (diff, borrow) = self.general_registers[y].overflowing_sub(self.general_registers[x]);
                self.general_registers[x] = diff;
                self.general_registers[0xF] = if borrow { 0 } else { 1 };
            },
            (8, _, _, 0xE) => {
                // Shift VX left by 1, set VF to MSB
                let x = nibble2 as usize;
                self.general_registers[0xF] = (self.general_registers[x] & 0x80) >> 7;
                self.general_registers[x] <<= 1;
            },
            (9, _, _, 0) => {
                // Skip next instruction if VX != VY
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                if self.general_registers[x] != self.general_registers[y] {
                    self.program_counter += 2;
                }
            },
            (0xA, _, _, _) => {
                // Set I to NNN
                self.index_register = opcode & 0x0FFF;
            },
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.program_counter = (self.general_registers[0] as u16) + nnn;
            },
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (opcode & 0xFF) as u8;
                let rng: u8 = random();
                self.general_registers[x] = rng & nn;
            },
            (0xD, _, _, _) => {
                // Display/draw a sprite
                let x = self.general_registers[nibble2 as usize] as usize % DISPLAY_WIDTH;
                let y = self.general_registers[nibble3 as usize] as usize % DISPLAY_HEIGHT;
                let height = nibble4;

                let mut flipped = false;
                for row in 0..height {
                    let sprite_byte = self.memory[(self.index_register + row as u16) as usize];
                    for col in 0..8 {
                        if (sprite_byte & (0x80 >> col)) != 0 {
                            let idx = (x + col) % DISPLAY_WIDTH + ((y + row as usize) % DISPLAY_HEIGHT) * DISPLAY_WIDTH;
                            flipped |= self.framebuffer[idx];
                            self.framebuffer[idx] ^= true;
                        }
                    }
                }
                self.general_registers[0xF] = if flipped { 1 } else { 0 };
            },
            // Skip if key pressed - EX9E
            (0xE, _, 9, 0xE) => {
                let x = nibble2 as usize;
                let vx = self.general_registers[x];
                let key = self.input_keys[vx as usize];
                if key {
                    self.program_counter += 2;
                }
            },

            // Skip if key not pressed - EXA1
            (0xE, _, 0xA, 1) => {
                let x = nibble2 as usize;
                let vx = self.general_registers[x];
                let key = self.input_keys[vx as usize];
                if !key {
                    self.program_counter += 2;
                }
            },

            // VX = DT - FX07
            (0xF, _, 0, 7) => {
                let x = nibble2 as usize;
                self.general_registers[x] = self.delay_timer;
            },

            // Wait Key Press - FX0A
            (0xF, _, 0, 0xA) => {
                let x = nibble2 as usize;
                let mut pressed = false;
                for i in 0..self.input_keys.len() {
                    if self.input_keys[i] {
                        self.general_registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    // Redo opcode
                    self.program_counter -= 2;
                }
            },
            // DT = VX - FX15
            (0xF, _, 1, 5) => {
                let x = nibble2 as usize;
                self.delay_timer = self.general_registers[x];
            },
            // ST = VX - FX18
            (0xF, _, 1, 8) => {
                let x = nibble2 as usize;
                self.sound_timer = self.general_registers[x];
            },
            // I += VX - FX1E
            (0xF, _, 1, 0xE) => {
                let x = nibble2 as usize;
                let vx = self.general_registers[x] as u16;
                self.index_register = self.index_register.wrapping_add(vx);
            },
            // I = FONT - FX29
            (0xF, _, 2, 9) => {
                let x = nibble2 as usize;
                let c = self.general_registers[x] as u16;
                self.index_register = c * 5;
            },
            // BCD of VX - FX33
            (0xF, _, 3, 3) => {
                let x = nibble2 as usize;
                let vx = self.general_registers[x] as f32;

                // Fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                self.memory[self.index_register as usize] = hundreds;
                self.memory[(self.index_register + 1) as usize] = tens;
                self.memory[(self.index_register + 2) as usize] = ones;
            },
            // STORE V0 - VX - FX55
            (0xF, _, 5, 5) => {
                let x = nibble2 as usize;
                let i = self.index_register as usize;
                for idx in 0..=x {
                    self.memory[i + idx] = self.general_registers[idx];
                }
            },
            // LOAD V0 - VX - FX65
            (0xF, _, 6, 5) => {
                let x = nibble2 as usize;
                let i = self.index_register as usize;
                for idx in 0..=x {
                    self.general_registers[idx] = self.memory[i + idx];
                }
            },
            (_,_,_,_) => unimplemented!("Not a valid opcode: {}",opcode),        }
    }



    /// Updates the delay and sound timers
    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
