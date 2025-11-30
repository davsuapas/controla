import Box from '@mui/material/Box';
import MarcajeList from './MarcajeList';
import { Marcaje } from '../modelos/marcaje';
import React, { useState } from 'react';
import { NetErrorControlado } from '../net/interceptor';
import CircularProgress from '@mui/material/CircularProgress';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import dayjs from 'dayjs';
import { Horario } from '../modelos/usuarios';
import useNotifications from '../hooks/useNotifications/useNotifications';
import Chip from '@mui/material/Chip';
import { api } from '../api/fabrica';
import { logError } from '../error';
import Backdrop from '@mui/material/Backdrop';

interface ResumenMarcajesProps {
  ultimosMarcajes: boolean,
  usuarioId?: string;
  fecha?: dayjs.Dayjs;
  horaInicio?: dayjs.Dayjs;
  refreshTrigger?: number;
}

// Muestra en una tabla los últimos marcajes de un usuario
// si la propiedad ultimos_marcajes es true, si no muestra
// el marcaje por usuario y fecha.
// También, muestra el horario más cercano si se proporciona
// una fecha y hora, si no se devuelve el horario según la fecha
export default function ResumenMacaje(props: ResumenMarcajesProps) {
  const [marcaje, setMarcaje] = useState<Marcaje[]>([]);
  const [horarios, setHorarios] = useState<Horario[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const notifica = useNotifications();

  // Carga los últimos marcajes (solo depende de usuarioId)
  const cargarMarcaje = React.useCallback(
    async (
      ultimosMarcajes: boolean,
      usuarioId: string,
      fecha: dayjs.Dayjs | undefined) => {
      setIsLoading(true);

      try {
        let MarcajesData: Marcaje[] = [];
        if (ultimosMarcajes || (!ultimosMarcajes && !fecha)) {
          MarcajesData = await api().marcajes.ultimosMarcajes(usuarioId);
        } else {
          MarcajesData = await api().marcajes.marcajesPorFecha(
            usuarioId, fecha!);
        }
        setMarcaje(MarcajesData);
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('resumen-marcaje.cargar.marcajes', error);
          notifica.show(
            'Error inesperado al cargar los últimos marcajes',
            {
              severity: 'error',
              autoHideDuration: 5000,
            },
          );
        }
        setMarcaje([]);
      }

      setIsLoading(false);
    }, []);

  // Carga los horarios (depende de usuarioId, fecha y horaInicio)
  const cargarHorarios = React.useCallback(
    async (
      usuarioId: string,
      fecha: dayjs.Dayjs | undefined,
      horaInicio: dayjs.Dayjs | undefined) => {

      let horario: Horario[] = [];

      try {
        if (fecha && horaInicio) {
          const hora: dayjs.Dayjs = fecha
            .set('hour', horaInicio.hour())
            .set('minute', horaInicio.minute());
          horario = await api().usuarios.horarioCercano(usuarioId, hora);
        } else {
          if (fecha) {
            horario = await api().usuarios.horarioSinAsignar(usuarioId, fecha);
          }
        }
        setHorarios(horario);
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('resumen-marcaje.cargar.horarios', error);
          notifica.show(
            'Error inesperado al cargar el horario',
            {
              severity: 'error',
              autoHideDuration: 5000,
            },
          );
        }
        setHorarios([]);
      }
    }, []);

  // Efecto para cargar marcajes (solo cuando cambia usuarioId)
  React.useEffect(() => {
    if (props.usuarioId) {
      cargarMarcaje(props.ultimosMarcajes, props.usuarioId, props.fecha);
    } else {
      setMarcaje([]);
    }
  }, [props.ultimosMarcajes, props.usuarioId, props.fecha, props.refreshTrigger]);

  // Efecto para cargar horarios (cuando cambia usuarioId o fechaHora)
  React.useEffect(() => {
    if (props.usuarioId) {
      cargarHorarios(props.usuarioId, props.fecha, props.horaInicio);
    } else {
      setHorarios([]);
    }
  }, [props.usuarioId, props.fecha, props.horaInicio]);

  return (
    <Box sx={{ flex: 1, overflow: 'hidden', height: '100%' }}>
      <Divider />
      <Stack spacing={2} sx={{ height: '100%', mt: 1.5 }}>
        <Box
          sx={{
            display: 'flex', alignItems: 'center',
            gap: 1, flexWrap: 'wrap'
          }}>
          <Typography variant="body1" sx={{ fontWeight: 'bold' }}>
            Horario/s posible/s:
          </Typography>

          {horarios.length === 0 ? (
            <Typography variant="body2" color="text.secondary"
              fontStyle="italic">
              No hay horarios disponibles o ya están asignados
            </Typography>
          ) : (
            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
              {horarios.map((horario) => (
                <Chip
                  key={horario.horarioToStr()}
                  label={horario.horarioToStr()}
                  size="medium"
                  variant="outlined"
                />
              ))}
            </Box>
          )}
        </Box>
        <Box sx={{
          flex: 1, position: 'relative', minHeight: 0
        }}>
          <Backdrop
            sx={{
              zIndex: (theme) => theme.zIndex.drawer + 1,
              position: 'absolute'
            }}
            open={isLoading}
          >
            <CircularProgress color="inherit" />
          </Backdrop>
          <MarcajeList marcajes={marcaje} />
        </Box>
      </Stack>
    </Box>
  );
}