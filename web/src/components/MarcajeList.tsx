import { alpha, styled } from '@mui/material/styles';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell, { tableCellClasses } from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import Paper from '@mui/material/Paper';

import { Theme } from "@mui/material/styles";
import { Marcaje } from '../modelos/marcaje';
import { dateToStr } from '../modelos/formatos';

const StyledTableCell = styled(TableCell)(({ theme }: { theme: Theme }) => ({
  [`&.${tableCellClasses.head}`]: {
    backgroundColor: theme.palette.common.black,
    color: theme.palette.common.white,
  },
  [`&.${tableCellClasses.body}`]: {
    fontSize: 14,
  },
}));

const StyledTableRow = styled(TableRow)(({ theme }: { theme: Theme }) => ({
  '&:nth-of-type(odd)': {
    backgroundColor: alpha(theme.palette.action.hover, 0.08),
  },
  '&:last-child td, &:last-child th': {
    border: 0,
  },
}));

interface MarcajeListProps {
  marcajes: Marcaje[];
}

// Muestra en una tabla los marcajes de un usuario
// Se proporciona una lista de marcajes de usuario
// Pemite excluir columnas a través de propiedades de configuración
export default function MarcajeList({ marcajes: marcajes }: MarcajeListProps) {
  return (
    <TableContainer component={Paper}
      sx={{ maxWidth: '100%', maxHeight: '50vh', overflow: 'auto' }}>
      <Table
        aria-label="customized table" stickyHeader
        sx={{ minWidth: 700 }}>
        <TableHead>
          <TableRow>
            <StyledTableCell align="center" colSpan={7}>
              MARCAJES
            </StyledTableCell>
          </TableRow>
          <TableRow>
            <StyledTableCell>FECHA</StyledTableCell>
            <StyledTableCell align="right">ENTRADA</StyledTableCell>
            <StyledTableCell align="right">SALIDA</StyledTableCell>
            <StyledTableCell align="right">HORA A ENTRAR</StyledTableCell>
            <StyledTableCell align="right">HORA A SALIR</StyledTableCell>
            <StyledTableCell align="right">HORAS TRABAJADAS</StyledTableCell>
            <StyledTableCell align="right">HORAS A TRABAJAR</StyledTableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {marcajes.map((marcaje) => (
            <StyledTableRow>
              <StyledTableCell component="th" scope="row">
                {dateToStr(marcaje.fecha)}
              </StyledTableCell>
              <StyledTableCell align="right">
                {marcaje.horaInicio}
              </StyledTableCell>
              <StyledTableCell
                align="right"
                sx={{
                  color: marcaje.horaFin ? 'inherit' : 'error.main'
                }}
              >
                {marcaje.horaFinToStr()}
              </StyledTableCell>
              <StyledTableCell align="right">
                {marcaje.horario!.horaInicio}
              </StyledTableCell>
              <StyledTableCell align="right">
                {marcaje.horario!.horaFin}
              </StyledTableCell>
              <StyledTableCell
                align="right"
                sx={{
                  color: marcaje.horaFin ? 'inherit' : 'error.main'
                }}
              >
                {marcaje.horaTrabajadasToStr()}
              </StyledTableCell>
              <StyledTableCell align="right">
                {marcaje.horario!.horasATrabajarToStr()}
              </StyledTableCell>
            </StyledTableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}