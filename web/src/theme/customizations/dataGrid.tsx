import { Theme } from '@mui/material/styles';
import { gridClasses } from '@mui/x-data-grid';
import type { DataGridComponents } from '@mui/x-data-grid/themeAugmentation';

/* eslint-disable import/prefer-default-export */
export const dataGridCustomizations: DataGridComponents<Theme> = {
  MuiDataGrid: {
    styleOverrides: {
      root: ({ theme }) => ({
        height: '100%',
        [`& .${gridClasses.columnHeader}, & .${gridClasses.cell}`]: {
          outline: 'transparent',
        },
        [`& .${gridClasses.columnHeader}:focus-within, & .${gridClasses.cell}:focus-within`]: {
          outline: 'none',
        },
        [`& .${gridClasses.row}:hover`]: {
          cursor: 'pointer',
        },
        '& .MuiDataGrid-columnHeader--filledGroup .MuiDataGrid-columnHeaderTitleContainer': {
          borderBottom: '2px solid',
          borderBottomColor: (theme.vars || theme).palette.divider,
        },
      }),
    },
  },
};

export const dataGridStyles = {
  marcarFila: (theme: Theme) => ({
    backgroundColor: theme.palette.info.light,
    borderLeft: `4px solid ${theme.palette.info.dark}`,
    '& .MuiDataGrid-cell': {
      fontWeight: 600,
      color: theme.palette.info.dark,
    },
    '&:hover': {
      backgroundColor: theme.palette.info.light,
    }
  })
};