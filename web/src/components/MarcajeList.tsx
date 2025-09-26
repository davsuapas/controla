import { alpha, styled } from '@mui/material/styles';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell, { tableCellClasses } from '@mui/material/TableCell';
import TableContainer from '@mui/material/TableContainer';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import Paper from '@mui/material/Paper';

import { Theme } from "@mui/material/styles";
import { Registro } from '../modelos/registro';
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

interface RegistroListProps {
  registros: Registro[];
}

// Muestra en una tabla el registro de un usuario
// Se proporciona una lista de registros de usuario
// Pemite excluir columnas a través de propiedades de configuración
export default function MarcajeList({ registros }: RegistroListProps) {
  return (
    <TableContainer component={Paper}
      sx={{ maxWidth: '100%', maxHeight: '50vh', overflow: 'auto' }}>
      <Table
        aria-label="customized table" size='small' stickyHeader
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
          {registros.map((registro) => (
            <StyledTableRow>
              <StyledTableCell component="th" scope="row">
                {dateToStr(registro.fecha)}
              </StyledTableCell>
              <StyledTableCell align="right">
                {registro.horaInicio}
              </StyledTableCell>
              <StyledTableCell
                align="right"
                sx={{
                  color: registro.horaFinToStr() == 'Sin especificar' ?
                    'error.main' : 'inherit'
                }}
              >
                {registro.horaFinToStr()}
              </StyledTableCell>
              <StyledTableCell align="right">
                {registro.horario!.horaInicio}
              </StyledTableCell>
              <StyledTableCell align="right">
                {registro.horario!.horaFin}
              </StyledTableCell>
              <StyledTableCell
                align="right"
                sx={{
                  color: registro.horaTrabajadasToStr() == 'Sin especificar' ?
                    'error.main' : 'inherit'
                }}
              >
                {registro.horaTrabajadasToStr()}
              </StyledTableCell>
              <StyledTableCell align="right">
                {registro.horario!.horasATrabajarToStr()}
              </StyledTableCell>
            </StyledTableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}