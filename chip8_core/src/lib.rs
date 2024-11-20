use rand::random;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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
0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; NUM_REGS],
    i_register: u16,
    sp: u16,
    stack: [u16 ; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc : START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; NUM_REGS],
            i_register: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_registers = [0; NUM_REGS];
        self.i_register = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }


    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        //Fetch
        let op = self.fetch();
        //Decode
        self.execute(op);
    }
    fn execute(&mut self,op:u16)
    {
        let one=(op&0xF000)>>12;
        let two=(op&0x0F00)>>8;
        let three=(op&0x00F0)>>4;
        let four=op&0x000F;
        match(one,two,three,four)
        {
           
            
            (1,_,_,_)=>{let nnn=op&0xFFF;self.pc=nnn;},//jump 
            (2,_,_,_)=>{let nnn=op&0xFFF;self.push(self.pc);self.pc=nnn;},//Call subroutine
            (3,_,_,_)=>{
                let x=two as usize;
                let nn=(op&0xFF) as u8;
                if self.v_registers[x]== nn
                {
                    self.pc+=2;
                }
            },//skip VX==NN
            (4,_,_,_)=>{
                let x=two as usize;
                let nn=(op&0xFF) as u8;
                if self.v_registers[x]!= nn
                {
                    self.pc+=2;
                }
            },//skep next if VX !=nn
            (5,_,_,_)=>{
                let x=two as usize;
                let y=three as usize;
                if self.v_registers[x]== self.v_registers[y]
                {
                    self.pc+=2;
                }
            },//skip next if VX==vy
            (6,_,_,_)=>{let x= two as usize;
            let nn=(op&0xFF) as u8;
            self.v_registers[x]=nn;
            }//vx=nn,
            (7,_,_,_)=>{let x= two as usize;
                let nn=(op&0xFF) as u8;
                self.v_registers[x]=self.v_registers[x].wrapping_add(nn);},//VX+=NN
            (8,_,_,_) => { let x=two as usize; let y =three as usize;
                self.v_registers[x]=self.v_registers[y];
            },//VX=VY   
            (0,0,0xE,0xE) => {let retaddr=self.pop(); self.pc=retaddr;},//return to addr
            (0,0,0xE,0) => {self.screen =[false;SCREEN_WIDTH*SCREEN_HEIGHT];}, //clear screen
            (0,0,0,0) => return,//nop
            (_,_,_,_) => unimplemented!("Not a valid opcode: {}",op),
            // VX |= VY
            (8, _, _, 1) => {
                let x = two as usize;
                let y = three as usize;
                self.v_registers[x] |= self.v_registers[y];
            },
            // VX &= VY
            (8, _, _, 2) => {
                let x = two as usize;
                let y = three as usize;
                self.v_registers[x] &= self.v_registers[y];
            },
            // VX ^= VY
            (8, _, _, 3) => {
                let x = two as usize;
                let y = three as usize;
                self.v_registers[x] ^= self.v_registers[y];
            },
            // VX += VY
            (8, _, _, 4) => {
                let x = two as usize;
                let y = three as usize;

                let (new_vx, carry) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },
            // VX -= VY
            (8, _, _, 5) => {
                let x = two as usize;
                let y = three as usize;

                let (new_vx, borrow) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },
            // VX >>= 1
            (8, _, _, 6) => {
                let x = two as usize;
                let lsb = self.v_registers[x] & 1;
                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = lsb;
            },
            // VX = VY - VX
            (8, _, _, 7) => {
                let x = two as usize;
                let y = three as usize;

                let (new_vx, borrow) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_registers[x] = new_vx;
                self.v_registers[0xF] = new_vf;
            },
            // VX <<= 1
            (8, _, _, 0xE) => {
                let x = two as usize;
                let msb = (self.v_registers[x] >> 7) & 1;
                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = msb;
            },
             // SKIP VX != VY
            (9, _, _, 0) => {
                let x = two as usize;
                let y = three as usize;
                if self.v_registers[x] != self.v_registers[y] {
                    self.pc += 2;
                }
            },
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_register = nnn;
            },
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_registers[0] as u16) + nnn;
            },
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = two as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_registers[x] = rng & nn;
            },
            
        }
    }
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }
            self.sound_timer -= 1;
        }
    }

}
