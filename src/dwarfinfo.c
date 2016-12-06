#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include <fcntl.h>
#include <dwarf.h>
#include <libdwarf.h>
#include <inttypes.h>

typedef unsigned char u8;

typedef struct {
    int fd;

    Dwarf_Debug dbg;
    Dwarf_Regtable3 reg_table;
    Dwarf_Addr row_pc;
} dwarf_info_t;

#define RBP 6
#define RSP 7
#define RIP 16

const char *regnames[] = {
    "rax",
    "rcx",
    "rdx",
    "rbx",
    "rsi",
    "rdi",
    "rbp",
    "rsp",
    "r8",
    "r9",
    "r10",
    "r11",
    "r12",
    "r13",
    "r14",
    "r15",
    "rip"
};

void emit_info(const char *reg,
               Dwarf_Small value_type,
               Dwarf_Signed offset_relevant,
               Dwarf_Signed register_num,
               Dwarf_Signed offset_or_block_len,
               Dwarf_Ptr block_ptr) {
    printf("register %s = ", reg);

    if (value_type == DW_EXPR_OFFSET) {
        if (register_num == DW_FRAME_CFA_COL3) {
            printf("cfa + %Ld\n", offset_or_block_len);
        } else if (register_num == DW_FRAME_SAME_VAL) {
            printf("old value\n");
        } else if (register_num == DW_FRAME_UNDEFINED_VAL) {
            printf("UNDEFINED\n");
        } else if (offset_relevant) {
            printf("%s + %Ld\n", regnames[register_num], offset_or_block_len);
        } else {
            printf("%s\n", regnames[register_num]);
        }

    } else {
        printf("UNKNOWN (value_type(%d) != DW_EXPR_OFFSET)\n", value_type);
    }
}

dwarf_info_t *dwarfinfo_init(const char *name, int32_t reg_table_size) {
    Dwarf_Error err;
    dwarf_info_t *info = calloc(sizeof(dwarf_info_t), 1);

    if (!info) {
        fprintf (stderr, "allocation failed\n");
        exit (EXIT_FAILURE);
    }

    if ((info->fd = open(name, O_RDONLY)) < 0) {
        fprintf (stderr, "cannot open file `%s`\n", name);
        exit (EXIT_FAILURE);
    }

    if (dwarf_init (info->fd, DW_DLC_READ, 0, 0, &info->dbg, &err) != DW_DLV_OK) {
        fprintf (stderr, "Failed DWARF initialization\n");
        exit (EXIT_FAILURE);
    }

    info->reg_table.rt3_reg_table_size = reg_table_size;
    info->reg_table.rt3_rules = calloc (sizeof(Dwarf_Regtable_Entry3),
                                        reg_table_size);

    if (!info->reg_table.rt3_rules) {
        fprintf (stderr, "allocation failed\n");
        exit (EXIT_FAILURE);
    }

    return info;
}

typedef struct {
    uintptr_t pc;
    uintptr_t fp;
    uintptr_t sp;
} machine_context_t;

static uintptr_t dwarfinfo_get_register (machine_context_t *mctxt, int regnum) {
    switch (regnum) {
        case RBP: return mctxt->fp;
        case RSP: return mctxt->sp;
        default: assert(false);
    }
}

static uintptr_t dwarfinfo_get_value (Dwarf_Regtable_Entry3 *entry, machine_context_t *mctxt,
                                      uintptr_t old_value, uintptr_t cfa) {
    Dwarf_Small value_type = entry->dw_value_type;
    Dwarf_Signed offset_relevant = entry->dw_offset_relevant;
    Dwarf_Signed register_num = entry->dw_regnum;
    Dwarf_Signed offset_or_block_len = entry->dw_offset_or_block_len;

    if (register_num == DW_FRAME_SAME_VAL) {
        return old_value;

    } else if (register_num == DW_FRAME_UNDEFINED_VAL) {
        assert (false);

    } else if (value_type == DW_EXPR_OFFSET) {
        if (offset_relevant) {
            if (register_num != DW_FRAME_CFA_COL3) {
                return dwarfinfo_get_register (mctxt, register_num) + offset_or_block_len;
            } else {
                uintptr_t *ptr = (uintptr_t*) (cfa + offset_or_block_len);

                return *ptr;
            }

        } else {
            return dwarfinfo_get_register (mctxt, register_num);
        }

    } else if (value_type == DW_EXPR_EXPRESSION) {
        assert (false && "DW_EXPR_EXPRESSION not yet supported.");

    } else {
        assert (false);
    }
}

#define MAX 32
intptr_t stack[MAX];

#define PUSH(val) do { \
                        if (top < stack_end) \
                            *top++ = (val); \
                        else \
                            assert (false && "stack overflow"); \
                  } while (0)

#define POP(val) do { \
                        if (top < stack_start) \
                            assert (false && "stack underflow"); \
                        else \
                            *top-- = (val); \
                  } while (0)

static void dwarfinfo_eval(u8 *block_start, size_t len) {
    u8 *block_end = block_start + len;
    u8 *ptr = block_start;

    intptr_t *stack_start = &stack[0];
    intptr_t *stack_end = &stack[MAX];
    intptr_t top = &stack[0];

    while (ptr < block_end) {
        switch (*ptr) {
            case DW_OP_lit0:
            case DW_OP_lit1:
            case DW_OP_lit2:
            case DW_OP_lit3:
            case DW_OP_lit4:
            case DW_OP_lit5:
            case DW_OP_lit6:
            case DW_OP_lit7:
            case DW_OP_lit8:
            case DW_OP_lit9:
            case DW_OP_lit10:
            case DW_OP_lit11:
            case DW_OP_lit12:
            case DW_OP_lit13:
            case DW_OP_lit14:
            case DW_OP_lit15:
            case DW_OP_lit16:
            case DW_OP_lit17:
            case DW_OP_lit18:
            case DW_OP_lit19:
            case DW_OP_lit20:
            case DW_OP_lit21:
            case DW_OP_lit22:
            case DW_OP_lit23:
            case DW_OP_lit24:
            case DW_OP_lit25:
            case DW_OP_lit26:
            case DW_OP_lit27:
            case DW_OP_lit28:
            case DW_OP_lit29:
            case DW_OP_lit30:
            case DW_OP_lit31:
                {
                    intptr_t value = *ptr - DW_OP_lit0;
                    PUSH (value);

                }
        }
    }
}

static void dwarfinfo_get (dwarf_info_t *info, machine_context_t *mctxt) {
    uintptr_t cfa = dwarfinfo_get_value (&info->reg_table.rt3_cfa_rule, mctxt, 0, 0);
    mctxt->pc = dwarfinfo_get_value (&info->reg_table.rt3_rules[RIP], mctxt, mctxt->pc, cfa);
    mctxt->fp = dwarfinfo_get_value (&info->reg_table.rt3_rules[RBP], mctxt, mctxt->fp, cfa);
    mctxt->sp = dwarfinfo_get_value (&info->reg_table.rt3_rules[RSP], mctxt, mctxt->sp, cfa);
}

bool dwarfinfo_at(dwarf_info_t *info, machine_context_t *mctxt, Dwarf_Addr pc) {
    Dwarf_Signed count = 0;
    Dwarf_Cie *cie_data = 0;
    Dwarf_Signed cie_count = 0;
    Dwarf_Fde *fde_data = 0;
    Dwarf_Signed fde_count = 0;
    Dwarf_Error err = 0;
    int fres = 0;

    fres = dwarf_get_fde_list_eh (info->dbg, &cie_data, &cie_count,
                                  &fde_data, &fde_count, &err);

    if (fres == DW_DLV_OK) {
        Dwarf_Fde myfde = 0;
        Dwarf_Addr low_pc = 0;
        Dwarf_Addr high_pc = 0;

        fres = dwarf_get_fde_at_pc (fde_data,
                                    pc,
                                    &myfde,
                                    &low_pc,
                                    &high_pc,
                                    &err);

        if (fres == DW_DLV_OK) {
            fres = dwarf_get_fde_info_for_all_regs3 (myfde,
                                                     pc,
                                                     &info->reg_table,
                                                     &info->row_pc,
                                                     &err);

            if (fres == DW_DLV_OK) {
                {
                    Dwarf_Regtable_Entry3 *entry = &info->reg_table.rt3_cfa_rule;

                    printf ("------------------------\n");
                    emit_info ("cfa",
                               entry->dw_value_type,
                               entry->dw_offset_relevant,
                               entry->dw_regnum,
                               entry->dw_offset_or_block_len,
                               entry->dw_block_ptr);

                    for (int i=0; i<info->reg_table.rt3_reg_table_size; i++) {
                        entry = &info->reg_table.rt3_rules[i];

                        emit_info (regnames[i],
                                   entry->dw_value_type,
                                   entry->dw_offset_relevant,
                                   entry->dw_regnum,
                                   entry->dw_offset_or_block_len,
                                   entry->dw_block_ptr);
                    }
                }

                dwarfinfo_get (info, mctxt);
                return true;

            } else {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
}

void dwarfinfo_free(dwarf_info_t *info) {
    Dwarf_Error err;

    free (info->reg_table.rt3_rules);

    if (dwarf_finish (info->dbg, &err) != DW_DLV_OK) {
        fprintf (stderr, "Failed DWARF finalization\n");
        exit (EXIT_FAILURE);
    }

    close(info->fd);
    free(info);
}
