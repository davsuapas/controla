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
export default function MarcajeList({ marcajes }: MarcajeListProps) {
  return (
    <TableContainer component={Paper}
      sx={{
        maxWidth: '100%',
        overflow: 'auto',
        height: '100%',
        boxShadow: 0,  // Quitar sombra para que se vea más limpio al pegar
      }}>
      <Table
        aria-label="customized table"
        sx={{ minWidth: 700 }}>
        <TableHead>
          <TableRow>
            {/* 2. Aplicamos position: sticky a las celdas */}
            <StyledTableCell
              sx={{
                position: 'sticky',
                top: 0, // Se pega al TOP del contenedor de scroll (el Box en ConsultaMarcaje)
                zIndex: 10,
                // Asegurar que el fondo es opaco, ya que al ser sticky,
                // las filas del body pasarían por debajo si no fuera opaco
                backgroundColor: 'background.default',
              }}
            >
              FECHA
            </StyledTableCell>
            <StyledTableCell
              align="right"
              sx={{
                position: 'sticky',
                top: 0,
                zIndex: 10,
                backgroundColor: 'background.default',
              }}
            >
              ENTRADA
            </StyledTableCell>
            <StyledTableCell
              align="right"
              sx={{
                position: 'sticky',
                top: 0,
                zIndex: 10,
                backgroundColor: 'background.default',
              }}
            >
              SALIDA
            </StyledTableCell>
            <StyledTableCell
              align="right"
              sx={{
                position: 'sticky',
                top: 0,
                zIndex: 10,
                backgroundColor: 'background.default',
              }}
            >
              HORA A ENTRAR
            </StyledTableCell>
            <StyledTableCell
              align="right"
              sx={{
                position: 'sticky',
                top: 0,
                zIndex: 10,
                backgroundColor: 'background.default',
              }}
            >
              HORA A SALIR
            </StyledTableCell>
            <StyledTableCell
              align="right"
              sx={{
                position: 'sticky',
                top: 0,
                zIndex: 10,
                backgroundColor: 'background.default',
              }}
            >
              HORAS TRABAJADAS
            </StyledTableCell>
            <StyledTableCell
              align="right"
              sx={{
                position: 'sticky',
                top: 0,
                zIndex: 10,
                backgroundColor: 'background.default',
              }}
            >
              HORAS A TRABAJAR
            </StyledTableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {marcajes.map((marcaje, index) => (
            <StyledTableRow key={index}>
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