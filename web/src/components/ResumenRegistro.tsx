import Box from '@mui/material/Box';
import MarcajeList from './MarcajeList';
import { Registro } from '../modelos/registro';
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

interface ResumenRegistrosProps {
  ultimosRegistros: boolean,
  usuarioId: string | undefined;
  fecha: dayjs.Dayjs | undefined;
  horaInicio: dayjs.Dayjs | undefined;
  refreshTrigger?: number;
}

// Muestra en una tabla los últimos registros de un usuario
// si la propierdad ultimos_registros es true, si no muestra
// el registro por usuario y fecha.
// También, muestra el horario más cercano si se proporciona
// una fecha y hora, si no se devuelve el horario según la fecha
export default function ResumenRegistros(props: ResumenRegistrosProps) {
  const [registros, setRegistros] = useState<Registro[]>([]);
  const [horarios, setHorarios] = useState<Horario[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const notifica = useNotifications();

  // Carga los últimos registros (solo depende de usuarioId)
  const cargarRegistros = React.useCallback(
    async (
      ultimosRegistros: boolean,
      usuarioId: string,
      fecha: dayjs.Dayjs | undefined) => {
      setIsLoading(true);

      try {
        let registrosData: Registro[] = [];
        if (ultimosRegistros || (!ultimosRegistros && !fecha)) {
          registrosData = await api().registros.ultimos_registros(usuarioId);
        } else {
          registrosData = await api().registros.marcajes_por_fecha(
            usuarioId, fecha!);
        }
        setRegistros(registrosData);
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('resumenregistro.cargar.registros', error);
          notifica.show(
            'Error inesperado al cargar los últimos marcajes',
            {
              severity: 'error',
              autoHideDuration: 5000,
            },
          );
        }
        setRegistros([]);
      }

      setIsLoading(false);
    }, []);

  // Carga los horarios (depende de usuarioId, fecha y horaInicio)
  // La horaIinicio trae la fecha de hoy, pero la que vale es la fecha 
  // el registrador asigna en el form
  const cargarHorarios = React.useCallback(
    async (
      usuarioId: string,
      fecha: dayjs.Dayjs | undefined,
      horaInicio: dayjs.Dayjs | undefined) => {

      let horario: Horario[] = [];

      setIsLoading(false);

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
          logError('resumenregistro.cargar.horarios', error);
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

  // Efecto para cargar registros (solo cuando cambia usuarioId)
  React.useEffect(() => {
    if (props.usuarioId) {
      cargarRegistros(props.ultimosRegistros, props.usuarioId, props.fecha);
    } else {
      setRegistros([]);
    }
  }, [props.ultimosRegistros, props.usuarioId, props.fecha, props.refreshTrigger]);

  // Efecto para cargar horarios (cuando cambia usuarioId o fechaHora)
  React.useEffect(() => {
    if (props.usuarioId) {
      cargarHorarios(props.usuarioId, props.fecha, props.horaInicio);
    } else {
      setHorarios([]);
    }
  }, [props.usuarioId, props.fecha, props.horaInicio]);

  if (isLoading) {
    return (
      <Box
        sx={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          width: '100%',
          m: 1,
        }}
      >
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box sx={{ flex: 1, overflow: 'hidden' }}>
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
        <Box sx={{ flex: 1, overflow: 'auto' }}>
          <MarcajeList registros={registros} />
        </Box>
      </Stack>
    </Box>
  );
}