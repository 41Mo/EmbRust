MEMORY
{
  /* FLASH and RAM are mandatory memory regions */

  /* STM32H742xI/743xI/753xI       */
  /* STM32H745xI/747xI/755xI/757xI */
  /* STM32H7A3xI/7B3xI             */
  FLASH  : ORIGIN = 0x08000000, LENGTH = 2M

  /* STM32H742xG/743xG       */
  /* STM32H745xG/STM32H747xG */
  /* STM32H7A3xG             */
  /* FLASH  : ORIGIN = 0x08000000, LENGTH = 512K */
  /* FLASH1 : ORIGIN = 0x08100000, LENGTH = 512K */

  /* STM32H750xB   */
  /* STM32H7B0     */
  /* FLASH  : ORIGIN = 0x08000000, LENGTH = 128K */

  /* DTCM  */
  RAM    : ORIGIN = 0x20000000, LENGTH = 128K

  /* AXISRAM */
  AXISRAM : ORIGIN = 0x24000000, LENGTH = 512K

  /* SRAM */
  SRAM1 : ORIGIN = 0x30000000, LENGTH = 288K
  SRAM2 : ORIGIN = 0x38000000, LENGTH = 64K

  /* Instruction TCM */
  ITCM  : ORIGIN = 0x00000000, LENGTH = 64K
}

/* The location of the stack can be overridden using the
   `_stack_start` symbol.  Place the stack at the end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* The location of the .text section can be overridden using the
   `_stext` symbol.  By default it will place after .vector_table */
/* _stext = ORIGIN(FLASH) + 0x40c; */

/* These sections are used for some of the examples */
SECTIONS {
  .axisram (NOLOAD) : ALIGN(8) {
    *(.axisram .axisram.*);
    . = ALIGN(8);
    } > AXISRAM
};
