import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import CircularProgress from '@mui/material/CircularProgress';
import Checkbox from '@mui/material/Checkbox';
import FormControlLabel from '@mui/material/FormControlLabel';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import { api } from '../api/fabrica';
import { Config, Localizacion } from '../modelos/config';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { logError } from '../error';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import { useIsMounted } from '../hooks/useComponentMounted';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import useConfig from '../hooks/useConfig/useConfig';


// Límites válidos para el radio de tolerancia del recinto (metros).
// Se usan tanto para acotar el valor sugerido por defecto como para
// validar lo que el usuario introduzca en el formulario.
const MARGEN_RECINTO_MIN = 50;
const MARGEN_RECINTO_MAX = 1000;

// Factor de colchón aplicado sobre la precisión (accuracy) del GPS
// para sugerir un radio de tolerancia por defecto razonable.
const FACTOR_MARGEN_TOLERANCIA = 1.5;

const MAP_URL = (localizacion: Localizacion) => {
  // 1 grado de latitud ≈ 111320 metros (aprox. constante en todo el planeta)
  const METROS_POR_GRADO = 111320;

  // Margen para que el área de precisión no ocupe todo el recuadro (se ve mejor con contexto alrededor)
  const FACTOR_MARGEN = 2.5;

  // Límites para evitar zooms absurdos (muy cerca o demasiado lejos)
  const DELTA_MIN = 0.0008; // ~90 m de lado, buen zoom para precisión alta
  const DELTA_MAX = 0.02;   // ~2.2 km de lado, tope para precisión muy baja

  const deltaCalculado = (localizacion.accuracy * FACTOR_MARGEN) / METROS_POR_GRADO;
  const delta = Math.min(Math.max(deltaCalculado, DELTA_MIN), DELTA_MAX);

  const minLon = localizacion.lng - delta;
  const minLat = localizacion.lat - delta;
  const maxLon = localizacion.lng + delta;
  const maxLat = localizacion.lat + delta;

  return `https://www.openstreetmap.org/export/embed.html?bbox=${minLon},${minLat},${maxLon},${maxLat}&layer=mapnik&marker=${localizacion.lat},${localizacion.lng}`;
};

// Calcula un radio de tolerancia sugerido a partir de la precisión (accuracy)
// reportada por el GPS, acotado siempre al rango válido del formulario.
const calcularMargenSugerido = (accuracy: number) => {
  const margenCalculado = accuracy * FACTOR_MARGEN_TOLERANCIA;
  return Math.round(
    Math.min(Math.max(margenCalculado, MARGEN_RECINTO_MIN), MARGEN_RECINTO_MAX)
  );
};

export default function ConfigForm() {
  const notifica = useNotifications();
  const dialogo = useDialogs();
  const isMounted = useIsMounted();

  const [isLoading, setIsLoading] = React.useState(true);
  const [config, setConfig] = React.useState<Config | null>(null);
  const [acotarMarcajes, setAcotarMarcajes] = React.useState(false);
  const [margenRecintoError, setMargenRecintoError] = React.useState<string | undefined>();
  const configGlobal = useConfig();

  const loadData = React.useCallback(async () => {
    setIsLoading(true);

    try {
      const data = await api().config.data();

      if (isMounted.current) {
        setConfig(data);
        setAcotarMarcajes(!!data.localizacion);
      }
    } catch (err) {
      if (!(err instanceof NetErrorControlado)) {
        logError('config-form.cargar', dialogo?.alert, err);
        if (isMounted.current) {
          notifica.show('Error inesperado al cargar la configuración', {
            severity: 'error',
            autoHideDuration: 5000,
          });
        }
      }
    } finally {
      if (isMounted.current) {
        setIsLoading(false);
      }
    }
  }, [isMounted, dialogo, notifica]);

  React.useEffect(() => {
    loadData();
  }, [loadData]);

  const handleToggleAcotar = React.useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const checked = event.target.checked;

      if (!checked) {
        setAcotarMarcajes(false);
        setConfig(prev => prev ? new Config(
          { ...prev, localizacion: null, margenRecinto: null }) : prev);
        setMargenRecintoError(undefined);
        return;
      }

      setAcotarMarcajes(true);
      setConfig(prev => prev ?
        new Config({ ...prev, margenRecinto: 100 }) :
        new Config({ margenRecinto: 100 }));
      setMargenRecintoError(undefined);

      if (!('geolocation' in navigator)) {
        notifica.show('Necesito permisos para obtener la ubicación', {
          severity: 'warning',
          autoHideDuration: 5000,
        });
        return;
      }

      try {
        const position = await new Promise<GeolocationPosition>((resolve, reject) => {
          navigator.geolocation.getCurrentPosition(resolve, reject, {
            enableHighAccuracy: true,
            timeout: 10000,
            maximumAge: 0,
          });
        });

        const nuevaLocalizacion: Localizacion = {
          lat: position.coords.latitude,
          lng: position.coords.longitude,
          accuracy: position.coords.accuracy,
        };

        const margenSugerido = calcularMargenSugerido(nuevaLocalizacion.accuracy);

        setConfig(prev => prev
          ? new Config({ ...prev, localizacion: nuevaLocalizacion, margenRecinto: margenSugerido })
          : new Config({ localizacion: nuevaLocalizacion, margenRecinto: margenSugerido }));
      } catch (error: any) {
        const message = error?.code === error?.PERMISSION_DENIED
          ? 'Necesito permisos para obtener la ubicación'
          : 'No se pudo obtener la ubicación';
        notifica.show(message, {
          severity: 'error',
          autoHideDuration: 5000,
        });
        setAcotarMarcajes(false);
      }
    }, [notifica]);

  const handleMargenRecintoChange = React.useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    const value = Number(event.target.value);
    setConfig(prev => prev ? new Config({ ...prev, margenRecinto: value }) : new Config({ margenRecinto: value }));
    setMargenRecintoError(undefined);
  }, []);

  const handleSubmit = React.useCallback(async () => {
    if (!config) return;

    if (acotarMarcajes &&
      (config.margenRecinto === null ||
        config.margenRecinto < MARGEN_RECINTO_MIN || config.margenRecinto > MARGEN_RECINTO_MAX)) {
      setMargenRecintoError(
        `El radio de tolerancia debe estar entre ${MARGEN_RECINTO_MIN} y ${MARGEN_RECINTO_MAX} metros`);
      return;
    }

    try {
      await api().config.actualizar(config);
      configGlobal.setConfig(config);
      notifica.show('Configuración actualizada satisfactoriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });
    } catch (err) {
      if (err instanceof NetErrorControlado) return;
      logError('config-form.actualizar', dialogo?.alert, err);
      notifica.show('Error inesperado al actualizar la configuración', {
        severity: 'error',
        autoHideDuration: 5000,
      });
    }
  }, [config, acotarMarcajes, dialogo, notifica]);

  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <PageContainer
      title="Configuración general"
      breadcrumbs={[{ title: 'Configuración', path: '/configuracion' }]}
    >
      <Box sx={{
        ...FULL_HEIGHT_WIDTH,
        display: 'flex',
        flexDirection: 'column',
        gap: 2,
        maxWidth: 720
      }}>
        <Paper sx={{ p: 3 }}>
          <Stack spacing={2}>
            <FormControlLabel
              control={
                <Checkbox
                  checked={acotarMarcajes}
                  onChange={handleToggleAcotar}
                />
              }
              label="Ubicar recinto para el control del marcaje por geo-localización"
            />
            <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
              Si habilita esta opción, tendrá que comunicar a sus empleados, que solo podrán realizar marcajes dentro del recinto de trabajo, o cerca de él. También es necesario que comunique a el empleado, que el sistema, cada vez que realice un marcaje, el empleado tiene que dar su permiso para que la aplicación obtenga su ubicación. <strong>NUNCA</strong> la ubicación del empleado se almacenará en la base de datos.
            </Typography>

            {acotarMarcajes && (
              <Stack spacing={1}>
                <TextField
                  name="margenRecinto"
                  label="Radio de tolerancia (m):"
                  type="number"
                  value={config?.margenRecinto ?? ''}
                  onChange={handleMargenRecintoChange}
                  error={!!margenRecintoError}
                  helperText={margenRecintoError ?? ' '}
                  fullWidth
                  slotProps={{
                    htmlInput: { min: MARGEN_RECINTO_MIN, max: MARGEN_RECINTO_MAX, step: 1 }
                  }}
                />
                {config?.localizacion && (
                  <Paper variant="outlined" sx={{ p: 1 }}>
                    <Typography variant="caption" sx={{ display: 'block', mb: 1 }}>
                      Precisión de la ubicación (±{Math.round(config.localizacion.accuracy)} m)
                    </Typography>
                    <Box
                      component="iframe"
                      src={MAP_URL(config.localizacion)}
                      title="Mapa de localización"
                      sx={{ width: '100%', height: 220, border: 0, borderRadius: 1 }}
                    />
                  </Paper>
                )}
              </Stack>
            )}

            <Button variant="contained" onClick={handleSubmit}>
              GUARDAR
            </Button>
          </Stack>
        </Paper>
      </Box>
    </PageContainer >
  );
}