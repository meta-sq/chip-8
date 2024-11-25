# Chip-8 Emulator

Chip-8 is a virtual environment originally developed in the 1970s, primarily used for developing video games. Over the years, many versions and updates to the original Chip-8 system have been introduced, such as CHIP-10, Hi-Res CHIP-8, CHIP-8C, and CHIP-8X. This project implements a Chip-8 emulator.

---

## Installation and Running Games

Follow these steps to set up and run Chip-8 games:

1. **Install SDL2 Library** (For Linux Only)

   Run the following command to install the SDL2 library if it is not already installed:
   ```bash
   sudo apt-get install libsdl2-dev
   ```

3. **Download Source Code**  
   Download the source code as a ZIP file from the [GitHub repository](#).

4. **Extract ZIP File**  
   Extract the downloaded ZIP file to your desired location.

5. **Navigate to Project Directory**  
   Open a terminal and navigate to the `desktop` directory inside the extracted folder:
   ```bash
   cd chip-8-main/desktop
   ```
   *Note:* If you changed the name of the parent folder during extraction, use that name instead of `chip-8-main`.

6. **Run a Game**  
   Use the following command to run a Chip-8 game:
   ```bash
   cargo run ../c8games/<name-of-game>
   ```
   For example, to run the game `PONG2`:
   ```bash
   cargo run ../c8games/PONG2
   ```

---

Feel free to explore the emulator and enjoy the retro gaming experience with Chip-8!
