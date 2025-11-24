import ResumenMarcaje from './ResumenMarcaje';
import Grid from '@mui/material/Grid';
import useNotifications from '../hooks/useNotifications/useNotifications';
import Button from '@mui/material/Button';
import { Box } from '@mui/material';
import dayjs from 'dayjs';
import React from 'react';
import { NetErrorControlado } from '../net/interceptor';
import { logError } from '../error';
import { api } from '../api/fabrica';
import { MarcajeOutDTO } from '../modelos/dto';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import SelectorEmpleado from './SelectorEmpleado';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';

// El marcaje automático se caracteríza por disponer de
// dos botones; uno para la entrada y otro para la salida
// El sistema se encarga de obtener las horas de forma automática
// Esta pantalla solo puede ser usada por empleados y registradores
export default function MarcajeAuto() {
  const [fechaActual, setFechaActual] = React.useState<dayjs.Dayjs>(dayjs());
  const [entrada, setEntrada] = React.useState<boolean>(true);
  const [bloquear, setBloquear] = React.useState<boolean>(false);

  const usuarioLog = useUsuarioLogeado().getUsrLogeado();
  const notifica = useNotifications();

  const usuarioSoloEmpleado = usuarioLog.tieneRol(RolID.Empleado) &&
    !usuarioLog.tieneRol(RolID.Registrador);

  const [empleado, setEmpleado] = React.useState<number>(usuarioLog.id);

  // Activa la entrada si todos los marcajes estan finalizados
  // Si hay una marcaje no finalizado se habilita la salida
  const activarEntradaSalida = React.useCallback(async () => {
    try {
      const fechaActual = dayjs();

      const salidaNula =
        await api().marcajes.marcajeSinFinalizar(empleado, fechaActual);

      setEntrada(!salidaNula)
      setBloquear(false);
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcaje-auto.salida-nula', error);
        notifica.show(
          'Error inesperado obteniendo estado del marcaje. ' +
          'Vuelva a intentarlo, recargando la página con F5', {
          severity: 'error',
          autoHideDuration: 8000,
        });
      }

      setBloquear(true);
    }
  }, []);

  React.useEffect(() => {
    activarEntradaSalida();
  }, [empleado]);

  // Marcar la entrada
  const handleEntrada = React.useCallback(async () => {
    try {
      const fechaActual = dayjs()

      await api().marcajes.registrar(
        MarcajeOutDTO.new(
          empleado,
          usuarioLog.toDescriptor(),
          fechaActual,
          fechaActual,
        )
      );

      setEntrada(false);
      setFechaActual(fechaActual);

      notifica.show('Entrada registrada satisfactóriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcaje-auto.entrada', error);
        notifica.show(
          'Error inesperado enviando la entrada del marcaje. ' +
          'Vuelva a intentarlo.', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    }
  }, []);

  // Marcar la salida
  const handleSalida = React.useCallback(async () => {
    try {
      const fechaActual = dayjs()

      await api().marcajes.registrarSalida(empleado, fechaActual)

      setEntrada(true);
      setFechaActual(fechaActual);

      notifica.show('Salida registrada satisfactóriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcaje-auto.salida', error);
        notifica.show(
          'Error inesperado enviando la salida del marcaje. ' +
          'Vuelva a intentarlo.', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    }
  }, []);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleado(empleado.id);
    },
    []
  );

  return (
    <PageContainer title={'Marcaje automático del empleado'}>
      <Box sx={{ ...FULL_HEIGHT_WIDTH }}>
        {!usuarioSoloEmpleado && (
          <>
            <SelectorEmpleado
              onChange={handleEmpleadoChange}
              onLoadingChange={setBloquear}
              usuarioPorDefecto={usuarioLog.id}
            />
            <Box sx={{ mb: 3 }} />
          </>
        )}
        <Grid container spacing={2} sx={{ mb: 3 }}>
          <Grid size={{ xs: 12, sm: 12, md: 4 }}>
            <Button
              variant="contained"
              color="success"
              fullWidth
              size="large"
              disabled={bloquear || !entrada}
              onClick={handleEntrada}
            >
              ENTRADA
            </Button>
          </Grid>
          <Grid size={{ xs: 12, sm: 12, md: 4 }}>
            <Button
              variant="contained"
              color="error"
              fullWidth
              size="large"
              disabled={bloquear || entrada}
              onClick={handleSalida}
            >
              SALIDA
            </Button>
          </Grid>
        </Grid>
        <ResumenMarcaje
          ultimosMarcajes={true}
          usuarioId={empleado.toString()}
          fecha={fechaActual}
          horaInicio={fechaActual}
        />
      </Box>
    </PageContainer>
  );
}