import ResumenMarcaje from './MarcajeResumen';
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
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { useIsMounted } from '../hooks/useComponentMounted';
import useConfig from '../hooks/useConfig/useConfig';
import { Config } from '../modelos/config';

// Distancia en metros entre dos coordenadas geográficas (fórmula del semiverseno)
function calcDistanciaMetros(
  lat1: number, lng1: number, lat2: number, lng2: number
): number {
  const R = 6371e3;
  const dLat = (lat2 - lat1) * Math.PI / 180;
  const dLng = (lng2 - lng1) * Math.PI / 180;
  const a = Math.sin(dLat / 2) ** 2 +
    Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) *
    Math.sin(dLng / 2) ** 2;
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}

// El marcaje automático se caracteríza por disponer de
// dos botones; uno para la entrada y otro para la salida
// El sistema se encarga de obtener las horas de forma automática
// Esta pantalla solo puede ser usada por empleados y registradores
export default function MarcajeAuto() {
  const usuarioLog = useUsuarioLogeado().getUsrLogeado();
  const notifica = useNotifications();
  const dialogo = useDialogs();
  const isMounted = useIsMounted();

  const [fechaActual, setFechaActual] = React.useState<dayjs.Dayjs>(dayjs());
  const [entrada, setEntrada] = React.useState<boolean>(true);
  const [bloquear, setBloquear] = React.useState<boolean>(false);
  const [ubicacionValida, setUbicacionValida] = React.useState<boolean | null>(true);

  const { getConfig } = useConfig();
  const config = React.useMemo<Config | null>(() => {
    try { return getConfig(); }
    catch { return null; }
  }, [getConfig]);

  const comprobarUbicacion = React.useCallback(async (): Promise<boolean> => {
    if (!config?.localizacion) return true;

    if (!('geolocation' in navigator)) {
      notifica.show(
        'La geolocalización no está disponible en este navegador. ' +
        'No se pueden realizar marcajes sin acceso a la ubicación.',
        { severity: 'error', autoHideDuration: 10000 }
      );
      return false;
    }

    const margen = config.margenRecinto ?? 0;

    try {
      const position = await new Promise<GeolocationPosition>((resolve, reject) => {
        navigator.geolocation.getCurrentPosition(resolve, reject, {
          enableHighAccuracy: true,
          timeout: 10000,
          maximumAge: 0,
        });
      });

      const distancia = calcDistanciaMetros(
        config.localizacion!.lat, config.localizacion!.lng,
        position.coords.latitude, position.coords.longitude
      );

      const valida = distancia <= margen;
      setUbicacionValida(valida);

      if (!valida) {
        notifica.show(
          'No se pueden realizar marcajes si no se encuentra ' +
          'dentro del edificio de trabajo. Distancia: ' + distancia.toString(),
          { severity: 'warning', autoHideDuration: 8000 }
        );
      }

      return valida;
    } catch (error: any) {
      setUbicacionValida(false);
      const message = error?.code === error?.PERMISSION_DENIED
        ? 'Permiso de geolocalización denegado. ' +
        'No se pueden realizar marcajes sin acceso a la ubicación.'
        : error?.code === error?.POSITION_UNAVAILABLE
          ? 'No se pudo obtener la ubicación. ' +
          'Señal GPS no disponible. No se pueden realizar marcajes.'
          : error?.code === error?.TIMEOUT
            ? 'La solicitud de ubicación ha expirado. ' +
            'No se pueden realizar marcajes.'
            : 'Error inesperado de geolocalización. ' +
            'No se pueden realizar marcajes.';
      notifica.show(message, { severity: 'error', autoHideDuration: 10000 });
      return false;
    }
  }, [config, notifica]);

  const usuarioSoloEmpleado = !usuarioLog.tieneRol(RolID.Registrador);

  const [empleado, setEmpleado] = React.useState<number>(usuarioLog.id);

  // Activa la entrada si todos los marcajes estan finalizados
  // Si hay una marcaje no finalizado se habilita la salida
  const activarEntradaSalida = React.useCallback(async () => {
    try {
      const fechaActual = dayjs();

      const salidaNula =
        await api().marcajes.marcajeSinFinalizar(empleado, fechaActual);

      if (isMounted.current) {
        setEntrada(!salidaNula)
        setBloquear(false);
      };

    } catch (error) {
      // El manejo de errores puede o no usar safeExecute.
      // Las notificaciones (notifica.show) generalmente se pueden disparar
      // sin necesidad de safeExecute, ya que no suelen actualizar el estado interno
      // del componente que se está desmontando, sino un componente externo (el Notifier).      if (!(error instanceof NetErrorControlado)) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcaje-auto.salida-nula', dialogo?.alert, error);
        notifica.show(
          'Error inesperado obteniendo estado del marcaje. ' +
          'Vuelva a intentarlo, recargando la página con F5', {
          severity: 'error',
          autoHideDuration: 8000,
        });
      }

      if (isMounted.current) {
        setBloquear(true);
      };
    }
  }, [empleado, notifica, dialogo]);

  React.useEffect(() => {
    activarEntradaSalida();
  }, [empleado, activarEntradaSalida]);

  // Marcar la entrada
  const handleEntrada = React.useCallback(async () => {
    if (!await comprobarUbicacion()) return;

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

      if (isMounted.current) {
        setEntrada(false);
        setFechaActual(fechaActual);
      };

      notifica.show('Entrada registrada satisfactóriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcaje-auto.entrada', dialogo?.alert, error);
        notifica.show(
          'Error inesperado enviando la entrada del marcaje. ' +
          'Vuelva a intentarlo.', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    }
  }, [notifica, dialogo, comprobarUbicacion]);

  // Marcar la salida
  const handleSalida = React.useCallback(async () => {
    if (!await comprobarUbicacion()) return;

    try {
      const fechaActual = dayjs()

      await api().marcajes.registrarSalida(empleado, fechaActual)

      if (isMounted.current) {
        setEntrada(true);
        setFechaActual(fechaActual);
      };

      notifica.show('Salida registrada satisfactóriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('marcaje-auto.salida', dialogo?.alert, error);
        notifica.show(
          'Error inesperado enviando la salida del marcaje. ' +
          'Vuelva a intentarlo.', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    }
  }, [notifica, dialogo, comprobarUbicacion]);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleado(empleado.id);
    },
    []
  );

  return (
    <PageContainer title={'Marcaje empleado'}>
      <Box sx={{
        ...FULL_HEIGHT_WIDTH, display: 'flex', flexDirection: 'column'
      }}>
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
              disabled={bloquear || !entrada || !ubicacionValida}
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
              disabled={bloquear || entrada || !ubicacionValida}
              onClick={handleSalida}
            >
              SALIDA
            </Button>
          </Grid>
        </Grid>
        <Box sx={{ flex: 1, minHeight: 300 }}>
          <ResumenMarcaje
            ultimosMarcajes={true}
            usuarioId={empleado.toString()}
            fecha={fechaActual}
            horaInicio={fechaActual}
          />
        </Box>
      </Box>
    </PageContainer>
  );
}