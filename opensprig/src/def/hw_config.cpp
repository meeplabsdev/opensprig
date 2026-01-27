#include <hw_config.h>

static spi_t spi = {.hw_inst = spi0,
                    .miso_gpio = 16,
                    .mosi_gpio = 19,
                    .sck_gpio = 18,
                    .baud_rate = 23950000,
                    .no_miso_gpio_pull_up = true};

static sd_spi_if_t sd_spi_if = {.spi = &spi, .ss_gpio = 21};

static sd_card_t sd_card = {
    .type = SD_IF_SPI, .spi_if_p = {&sd_spi_if}, .use_card_detect = false};

size_t sd_get_num() {
    return 1;
}

sd_card_t *sd_get_by_num(size_t num) {
    if (num == 0)
        return &sd_card;

    return NULL;
}
