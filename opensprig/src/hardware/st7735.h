// --------------------------------------------------------------------------
// ST7735S
//
// This code is based on work from Bernhard Bablok
// https://github.com/bablokb/pic-st7735
//
// The code is based on work from Gavin Lyons
// https://github.com/gavinlyonsrepo/pic_16F18346_projects
// --------------------------------------------------------------------------

#include <hardware/gpio.h>
#include <hardware/spi.h>
#include <initializer_list>
#include <pico/time.h>

#define NOP3(exp)                                                              \
    asm volatile("nop \n nop \n nop");                                         \
    exp;                                                                       \
    asm volatile("nop \n nop \n nop");

#define ST7735_NOP 0x00
#define ST7735_SWRESET 0x01
#define ST7735_RDDID 0x04
#define ST7735_RDDST 0x09
#define ST7735_SLPIN 0x10
#define ST7735_SLPOUT 0x11
#define ST7735_PTLON 0x12
#define ST7735_NORON 0x13
#define ST7735_INVOFF 0x20
#define ST7735_INVON 0x21
#define ST7735_DISPOFF 0x28
#define ST7735_DISPON 0x29
#define ST7735_CASET 0x2A
#define ST7735_RASET 0x2B
#define ST7735_RAMWR 0x2C
#define ST7735_RAMRD 0x2E
#define ST7735_PTLAR 0x30
#define ST7735_VSCRDEF 0x33
#define ST7735_COLMOD 0x3A
#define ST7735_MADCTL 0x36
#define ST7735_MADCTL_MY 0x80
#define ST7735_MADCTL_MX 0x40
#define ST7735_MADCTL_MV 0x20
#define ST7735_MADCTL_ML 0x10
#define ST7735_MADCTL_RGB 0x00
#define ST7735_VSCRSADD 0x37
#define ST7735_FRMCTR1 0xB1
#define ST7735_FRMCTR2 0xB2
#define ST7735_FRMCTR3 0xB3
#define ST7735_INVCTR 0xB4
#define ST7735_DISSET5 0xB6
#define ST7735_PWCTR1 0xC0
#define ST7735_PWCTR2 0xC1
#define ST7735_PWCTR3 0xC2
#define ST7735_PWCTR4 0xC3
#define ST7735_PWCTR5 0xC4
#define ST7735_VMCTR1 0xC5
#define ST7735_RDID1 0xDA
#define ST7735_RDID2 0xDB
#define ST7735_RDID3 0xDC
#define ST7735_RDID4 0xDD
#define ST7735_PWCTR6 0xFC
#define ST7735_GMCTRP1 0xE0
#define ST7735_GMCTRN1 0xE1

#define PORT spi0
#define RX 16
#define TX 19
#define CLK 18
#define CS 20
#define DC 22
#define RST 26

#define cs_low() NOP3(gpio_put(CS, 0));
#define cs_high() NOP3(gpio_put(CS, 1));

#define dc_low() NOP3(gpio_put(DC, 0));
#define dc_high() NOP3(gpio_put(DC, 1));

#define rst_low() NOP3(gpio_put(RST, 0));
#define rst_high() NOP3(gpio_put(RST, 1));

static void spi_command(uint8_t x) {
    dc_low();
    spi_write_blocking(PORT, &x, sizeof(x));
}

inline void spi_data(std::initializer_list<uint8_t> args) {
    dc_high();
    uint8_t data[args.size()];
    int i = 0;
    for (uint8_t v : args) {
        data[i++] = v;
    }
    spi_write_blocking(PORT, data, sizeof(data));
}

static void write_command(uint8_t cmd) {
    cs_low();
    spi_command(cmd);
    cs_high();
}

static void write_data(std::initializer_list<uint8_t> args) {
    for (uint8_t data : args) {
        cs_low();
        spi_data({data});
        cs_high();
    }
}

static void reset() {
    rst_high();
    sleep_ms(10);
    rst_low();
    sleep_ms(10);
    rst_high();
    sleep_ms(10);
}

static void fill_start() {
    cs_low();

    spi_command(ST7735_CASET);
    spi_data({0x00, 0x00, 0x00, 0x7F});

    spi_command(ST7735_RASET);
    spi_data({0x00, 0x00, 0x00, 0x9F});

    spi_command(ST7735_RAMWR);
    dc_high(); // for empty data
}

static void fill_send(uint16_t pixel) {
    spi_write_blocking(PORT, (uint8_t *)&pixel, sizeof(uint16_t));
}

static void fill_finish(void) {
    cs_high();
}

void init_screen_tft() {
    reset();
    dc_low();

    // read screen data sheet to understand
    {
        write_command(ST7735_SWRESET);
        sleep_ms(150);
        write_command(ST7735_SLPOUT);
        sleep_ms(500);
        write_command(ST7735_FRMCTR1);
        write_data({0x01, 0x2C, 0x2D});
        write_command(ST7735_FRMCTR2);
        write_data({0x01, 0x2D, 0x2D});
        write_command(ST7735_FRMCTR3);
        write_data({0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D});
        write_command(ST7735_INVCTR);
        write_data({0x07});
        write_command(ST7735_PWCTR1);
        write_data({0xA2, 0x02, 0x84});
        write_command(ST7735_PWCTR2);
        write_data({0xC5});
        write_command(ST7735_PWCTR3);
        write_data({0x0A, 0x00});
        write_command(ST7735_PWCTR4);
        write_data({0x8A, 0x2A});
        write_command(ST7735_PWCTR5);
        write_data({0x8A, 0xEE});
        write_command(ST7735_VMCTR1);
        write_data({0x0E});
        write_command(ST7735_INVOFF);
        write_command(ST7735_MADCTL);
        write_data({0x40 | 0x10 | 0x08}); // 0xC8
        write_command(ST7735_COLMOD);
        write_data({0x05});
    }

    {
        write_command(ST7735_CASET);
        write_data({0x00, 0x00, 0x00, 0x7F});
        write_command(ST7735_RASET);
        write_data({0x00, 0x00, 0x00, 0x9F});
    }

    {
        write_command(ST7735_GMCTRP1);
        write_data({0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D, 0x29, 0x25,
                    0x2B, 0x39, 0x00, 0x01, 0x03, 0x10});
        write_command(ST7735_GMCTRN1);
        write_data({0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E,
                    0x37, 0x3F, 0x00, 0x00, 0x02, 0x10});
        write_command(ST7735_NORON);
        sleep_ms(10);
        write_command(ST7735_DISPON);
        sleep_ms(100);
    }
}
