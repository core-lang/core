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

bool dwarfinfo_at(dwarf_info_t *info, Dwarf_Addr pc) {
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

            return fres == DW_DLV_OK;
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
