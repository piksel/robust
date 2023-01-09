/*	simple Hello World, for cc65, for NES
 *  writing to the screen with rendering disabled
 *	using neslib
 *	Doug Fraker 2018
 */	
 
 
 
#include "LIB/neslib.h"
#include "LIB/nesdoug.h" 
#include "logo_nametable.h"

#define BLACK 0x0f
#define DK_GY 0x00
#define LT_GY 0x10
#define WHITE 0x30
// there's some oddities in the palette code, black must be 0x0f, white must be 0x30
 
 
 
#pragma bss-name(push, "ZEROPAGE")

// GLOBAL VARIABLES
// all variables should be global for speed
// zeropage global is even faster

unsigned char b;
unsigned char f;

// unsigned char buf[] = {MSB(NTADR_A(10,8)), LSB(NTADR_A(10,8)), 0, NT_UPD_EOF};


const unsigned char text[]="No cart loaded"; // zero terminated c string

// const unsigned char palette[]={
// BLACK, DK_GY, LT_GY, WHITE,
// 0,0,0,0,
// 0,0,0,0,
// 0,0,0,0
// }; 

char palette[16]={
	// { 
		0x0f, 0x13, 0x03, 0x30,
		0x0f, 0x04, 0x05, 0x06,
		0x0f, 0x08, 0x09, 0x0a,
		0x0f, 0x13, 0x23, 0x30 
	// }
};

void delay_frames(unsigned char count) {
	for (f = 0; f <= count; f++) {
		ppu_wait_nmi();
	}
}

void set_palette_col(unsigned char b, char glow) {
	if (glow) {
		palette[0x1] = 0x20 | b;
		palette[0x2] = 0x10 | b;
		palette[0xd] = 0x20 | b;
		palette[0xe] = 0x30 | b;
	} else {
		palette[0x1] = 0x10 | b;
		palette[0x2] = 0x00 | b;
		palette[0xd] = 0x10 | b;
		palette[0xe] = 0x20 | b;
	}
	ppu_wait_nmi();
	pal_bg(palette);
}
	

void main (void) {
	
	ppu_off(); // screen off

	pal_bg(palette); //	load the BG palette
		
	// set a starting point on the screen
	// vram_adr(NTADR_A(x,y));
	// vram_adr(NTADR_A(10,14)); // screen is 32 x 30 tiles
	vram_adr(NTADR_A(0, 0));

	vram_unrle(NT_LOGO);

	vram_adr(NTADR_A(9,8)); // screen is 32 x 30 tiles
	vram_write(text,sizeof(text));
	
	ppu_on_all(); //	turn on screen
	

	
	while (1){
		for (b = 0x1; b <= 0x3; b++) {
			//set_palette_col(b, 0);
			//delay_frames(10);
			//scroll(b, 0);
		}

		



delay_frames(60);

		for (b = 0x3; b <= 0xc; b++) {
			set_palette_col(b, 0);
			delay_frames(10);
		}

		for (b = 0x1; b <= 0x3; b++) {
			set_palette_col(b, 0);
			delay_frames(10);
			//scroll(b, 0);
		}



delay_frames(60);

		for (b = 0; b <= 0x4; b++) {
			ppu_wait_nmi();
			scroll(0, b);
		}

		for (; b >= 1; b--) {
			ppu_wait_nmi();
			scroll(0, b);
		}

delay_frames(4);

		for (b = 0; b <= 0x4; b++) {
			ppu_wait_nmi();
			scroll(0, b);
		}

		for (; b >= 1; b--) {
			ppu_wait_nmi();
			scroll(0, b);
		}

delay_frames(30);

		for (b = 0; b <= 0x4; b++) {
			ppu_wait_nmi();
			scroll(0, b);
		}

		for (; b >= 1; b--) {
			ppu_wait_nmi();
			scroll(0, b);
		}

delay_frames(4);

		for (b = 0; b <= 0x4; b++) {
			ppu_wait_nmi();
			scroll(0, b);
		}

		for (; b >= 1; b--) {
			ppu_wait_nmi();
			scroll(0, b);
		}

		


delay_frames(60);

		for (b = 0; b <= 4; b++) {
			set_palette_col(3, 0);
			delay_frames(10);
			set_palette_col(3, 1);
			delay_frames(10);
			set_palette_col(3, 0);
		}
	}
}
	
	