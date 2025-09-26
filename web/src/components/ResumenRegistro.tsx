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
  usuarioId: string | undefined;
  fechaHora: dayjs.Dayjs | undefined;
  refreshTrigger?: number;
}

// Muestra en una tabla los últimos registros de un usuario
// y el horario más cercano si se proporciona una fecha y hora,
// si no se devuelve el horario del día no asignado
export default function ResumenRegistros(props: ResumenRegistrosProps) {
  const [registros, setRegistros] = useState<Registro[]>([]);
  const [horarios, setHorarios] = useState<Horario[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const notifica = useNotifications();

  // Carga los últimos registros (solo depende de usuarioId)
  const cargarRegistros = React.useCallback(async (usuarioId: string) => {
    setIsLoading(true);

    try {
      const registrosData = await api().registros.ultimos_marcajes(usuarioId);
      setRegistros(registrosData);
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('resumenregistro.cargar.registros', error);
        notifica.show(
          'Error inesperado al cargar los últimos registros',
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

  // Carga los horarios (depende de usuarioId y fechaHora)
  const cargarHorarios = React.useCallback(
    async (usuarioId: string, fechaHora: dayjs.Dayjs | undefined) => {
      setIsLoading(false);

      try {
        const horariosData = await api().usuarios.horario(usuarioId, fechaHora);
        setHorarios(horariosData);
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
      cargarRegistros(props.usuarioId);
    } else {
      setRegistros([]);
    }
  }, [props.usuarioId, props.refreshTrigger]);

  // Efecto para cargar horarios (cuando cambia usuarioId o fechaHora)
  React.useEffect(() => {
    if (props.usuarioId) {
      cargarHorarios(props.usuarioId, props.fechaHora);
    } else {
      setHorarios([]);
    }
  }, [props.usuarioId, props.fechaHora]);

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