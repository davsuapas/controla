import Box from '@mui/material/Box';
import { InformeCumplimiento } from '../modelos/informes';
import React, { useState } from 'react';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { logError } from '../error';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { api } from '../api/fabrica';
import { Backdrop, CircularProgress, Grid, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, useMediaQuery, useTheme, FormControl, InputLabel, Select, MenuItem, TextField, Typography } from '@mui/material';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';
import SelectorEmpleado from './SelectorEmpleado';
import { styled } from '@mui/material/styles';
import { alpha } from '@mui/material';
import { tableCellClasses } from '@mui/material/TableCell';
import dayjs from 'dayjs';
import { useIsMounted } from '../hooks/useComponentMounted';
import { dateToStr } from '../modelos/formatos';

const StyledTableCell = styled(TableCell)(({ theme }) => ({
  [`&.${tableCellClasses.head}`]: {
    backgroundColor: theme.palette.common.black,
    color: theme.palette.common.white,
  },
  [`&.${tableCellClasses.body}`]: {
    fontSize: 14,
  },
}));

const StyledTableRow = styled(TableRow)(({ theme }) => ({
  '&:nth-of-type(odd)': {
    backgroundColor: alpha(theme.palette.action.hover, 0.08),
  },
  '&:last-child td, &:last-child th': {
    border: 0,
  },
}));

const MONTH_NAMES = [
  "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
  "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre"
];

export default function InformeCumplimientoHorario() {
  const isMounted = useIsMounted();
  const theme = useTheme();
  const usuarioLog = useUsuarioLogeado().getUsrLogeado();
  const notifica = useNotifications();

  const [informe, setInforme] = useState<InformeCumplimiento | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const usuarioSoloEmpleado =
    !usuarioLog.tieneRol(RolID.Registrador) &&
    !usuarioLog.tieneRol(RolID.Supervisor);

  const [empleado, setEmpleado] = React.useState<number>(usuarioLog.id);
  const [mes, setMes] = React.useState<number>(dayjs().month() + 1); // 1-12
  const [anio, setAnio] = React.useState<number>(dayjs().year());

  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const [isExpanded, setIsExpanded] = React.useState<boolean>(false);

  const currentYear = dayjs().year();
  const currentMonth = dayjs().month() + 1;
  const years = Array.from({ length: 21 }, (_, i) => currentYear - i);

  React.useEffect(() => {
    if (anio === currentYear && mes > currentMonth) {
      setMes(currentMonth);
    }
  }, [anio, mes, currentYear, currentMonth]);

  const cargarInforme = React.useCallback(
    async () => {
      setIsLoading(true);

      let informeData: InformeCumplimiento | null = null;

      try {
        informeData = await api().informe.cumplimientoHorario(
          empleado, mes, anio);
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('informe-cumplimiento.cargar.informe', error);
          notifica.show(
            'Error inesperado al cargar el informe',
            {
              severity: 'error',
              autoHideDuration: 5000,
            },
          );
        }
      }

      if (isMounted.current) {
        setInforme(informeData);
        setIsLoading(false);
      }
    },
    [empleado, mes, anio, usuarioLog, notifica]
  );

  React.useEffect(() => {
    cargarInforme();
  }, [empleado, mes, anio, cargarInforme]);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleado(empleado.id);
    },
    []
  );

  const toggleExpand = React.useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  return (
    <PageContainer title={`Informe de Cumplimiento Horario (${MONTH_NAMES[mes - 1]} - ${anio})`}>
      <Box sx={{ ...FULL_HEIGHT_WIDTH, display: 'flex', flexDirection: 'column' }}>
        <Box sx={{
          display: isMobile && isExpanded ? 'none' : 'block',
          flexShrink: 0
        }}>
          {
            !usuarioSoloEmpleado && (
              <>
                <SelectorEmpleado
                  onChange={handleEmpleadoChange}
                  usuarioPorDefecto={usuarioLog.id}
                />
                <Box sx={{ mb: 3 }} />
              </>
            )
          }
          <Grid container spacing={2} sx={{ mt: 2, ml: 0.2, mb: 2, width: '100%' }}>
            <Grid size={{ xs: 12, sm: 6, md: 3 }}>
              <FormControl fullWidth>
                <InputLabel id="select-mes-label">Mes</InputLabel>
                <Select
                  labelId="select-mes-label"
                  value={mes}
                  label="Mes"
                  onChange={(e) => setMes(Number(e.target.value))}
                >
                  {MONTH_NAMES
                    .map((nombre, index) => ({ nombre, value: index + 1 }))
                    .filter(month =>
                      anio < currentYear || month.value <= currentMonth
                    )
                    .map(({ nombre, value }) => (
                      <MenuItem key={value} value={value}>
                        {nombre}
                      </MenuItem>
                    ))}
                </Select>
              </FormControl>
            </Grid>
            <Grid size={{ xs: 12, sm: 6, md: 3 }}>
              <FormControl fullWidth>
                <InputLabel id="select-anio-label">Año</InputLabel>
                <Select
                  labelId="select-anio-label"
                  value={anio}
                  label="Año"
                  onChange={(e) => setAnio(Number(e.target.value))}
                >
                  {years.map((year) => (
                    <MenuItem key={year} value={year}>
                      {year}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Grid>
          </Grid>
        </Box>
        {isMobile && (
          <Box
            onClick={toggleExpand}
            sx={{
              height: 20,
              flexShrink: 0,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              cursor: 'pointer',
              borderBottom: '1px solid',
              borderColor: 'divider',
              bgcolor: 'background.paper',
              '&:active': {
                bgcolor: 'action.hover'
              },
              transition: 'background-color 0.2s'
            }}
          >
            {isExpanded ? (
              <ExpandMoreIcon sx={{ color: 'text.secondary' }} />
            ) : (
              <ExpandLessIcon sx={{ color: 'text.secondary' }} />
            )}
          </Box>
        )}
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', alignItems: 'center', mb: 2, mr: 2 }}>
          <Typography variant="body2" sx={{ mr: 1, fontWeight: 'bold' }}>
            TOTAL SALDO (HORAS):
          </Typography>
          <TextField
            value={(informe?.totalSaldo ?? 0).toFixed(2)}
            hiddenLabel
            slotProps={{
              input: {
                readOnly: true,
                sx: {
                  '& input': {
                    textAlign: 'right',
                  },
                  color: (informe?.totalSaldo ?? 0) >= 0 ? 'success.main' : 'error.main',
                  fontWeight: 'bold'
                }
              }
            }}
            variant="outlined"
            size="small"
          />
        </Box>
        <Box sx={{ flex: 1, minHeight: 250, position: 'relative' }}>
          <Backdrop sx={{ zIndex: (theme) => theme.zIndex.drawer + 1, position: 'absolute' }} open={isLoading}>
            <CircularProgress color="inherit" />
          </Backdrop>
          <TableContainer component={Paper}
            sx={{ maxWidth: '100%', overflow: 'auto', height: '100%', boxShadow: 0 }}>
            <Table aria-label="customized table" sx={{ minWidth: 500 }}>
              <TableHead>
                <TableRow>
                  <StyledTableCell sx={{
                    width: '80px', position: 'sticky',
                    top: 0, zIndex: 10, backgroundColor: 'background.default'
                  }}>
                    FECHA
                  </StyledTableCell>
                  <StyledTableCell
                    align="left"
                    sx={{
                      width: '80px', position: 'sticky',
                      top: 0, zIndex: 10, backgroundColor: 'background.default'
                    }}>
                    DÍA
                  </StyledTableCell>
                  <StyledTableCell
                    align="right"
                    sx={{
                      width: '180px', position: 'sticky',
                      top: 0, zIndex: 10, backgroundColor: 'background.default'
                    }}>
                    HORAS TRABAJO EFECTIVO
                  </StyledTableCell>
                  <StyledTableCell
                    align="right"
                    sx={{
                      width: '180px', position: 'sticky',
                      top: 0, zIndex: 10, backgroundColor: 'background.default'
                    }}>
                    TOTAL HORAS TRABAJADAS
                  </StyledTableCell>
                  <StyledTableCell
                    align="right"
                    sx={{
                      width: '80px', position: 'sticky',
                      top: 0, zIndex: 10, backgroundColor: 'background.default'
                    }}>
                    HORAS A TRABAJAR
                  </StyledTableCell>
                  <StyledTableCell
                    align="right"
                    sx={{
                      width: '80px', position: 'sticky',
                      top: 0, zIndex: 10, backgroundColor: 'background.default'
                    }}>
                    SALDO
                  </StyledTableCell>
                  <StyledTableCell
                    align="left"
                    sx={{
                      position: 'sticky', top: 0, zIndex: 10,
                      backgroundColor: 'background.default',
                      whiteSpace: 'nowrap'
                    }}>
                    NOTAS
                  </StyledTableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {informe?.lineas.map((row, index) => (
                  <StyledTableRow key={index}>
                    <StyledTableCell component="th" scope="row">
                      {dateToStr(row.fecha)}
                    </StyledTableCell>
                    <StyledTableCell align="left">{row.getDiaSemana()}</StyledTableCell>
                    <StyledTableCell align="right">{row.horasTrabajoEfectivo.toFixed(2)}</StyledTableCell>
                    <StyledTableCell align="right">{row.horasTrabajadas.toFixed(2)}</StyledTableCell>
                    <StyledTableCell align="right">{row.horasATrabajar.toFixed(2)}</StyledTableCell>
                    <StyledTableCell
                      align="right"
                      sx={{
                        color: row.saldo >= 0 ? 'success.main' : 'error.main',
                        fontWeight: 'bold' // Opcional: para resaltar el saldo
                      }}
                    >
                      {row.saldo.toFixed(2)}
                    </StyledTableCell>
                    <StyledTableCell align="left" sx={{ whiteSpace: 'nowrap' }}>
                      {row.nota}
                    </StyledTableCell>
                  </StyledTableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Box>
      </Box>
    </PageContainer>
  );
}