import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import ButtonGroup from '@mui/material/ButtonGroup';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Grid';
import TextField from '@mui/material/TextField';
import {
  DataGrid,
  GridActionsCellItem,
  GridColDef,
  GridColumnGroupingModel,
  GridRowId,
} from '@mui/x-data-grid';
import AddIcon from '@mui/icons-material/Add';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import SelectorEmpleado from './SelectorEmpleado';
import { ConfigHorario, DescriptorUsuario, DiaSemana, diaSemanaToPalabra } from '../modelos/usuarios';
import { useLocation, useNavigate } from 'react-router';
import dayjs, { Dayjs } from 'dayjs';
import { dateToStr } from '../modelos/formatos';
import { api } from '../api/fabrica';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { NetErrorControlado } from '../net/interceptor';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import { useTheme } from '@mui/material/styles';
import { Tooltip } from '@mui/material';

// Componente para configurar los horarios de los empleados.
// Permite seleccionar un empleado, ver la última configuración,
// crear nuevas configuraciones, duplicar configuraciones existentes,
// y añadir, editar o eliminar horarios individuales.
// Aunque puede que haya múltiples configuraciónes, la última es 
// la que se encuentra vigente y es la única que se muestra
// Todos los horarios de una configuración tienen la misma fecha de
// creación
export default function HorarioConfigurador() {
  const theme = useTheme();
  const navigate = useNavigate();
  const location = useLocation();
  const notifica = useNotifications();
  const { confirm } = useDialogs();
  const isMounted = useIsMounted();
  const [fechaConfiguracion, setFechaConfiguracion] =
    React.useState<Dayjs>(dayjs().startOf('day'));

  const [isLoading, setIsLoading] = React.useState<boolean>(false);

  const [rows, setRows] = React.useState<ConfigHorario[]>([]);
  const [empleadoId, setEmpleadoId] = React.useState<number | null>(() => {
    const state = location.state as { usuarioId: number } | null;
    return state?.usuarioId ?? null;
  });

  const loadData = React.useCallback(async (id: number) => {
    setIsLoading(true);
    try {
      const data = await api().usuarios.horarios(id);
      if (isMounted.current) {
        setRows(data);
        if (data.length > 0) {
          // Todos tienen la misma fecha de configuración
          setFechaConfiguracion(data[0].fechaCreacion as Dayjs);
        } else {
          // Si no hay horarios es como si fuera una nueva
          // configuración.
          handleNuevaConfiguracion();
        }
      }
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('horario-configurador.cargar', error);
        notifica.show('Error inesperado al cargar los horarios', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    } finally {
      if (isMounted.current) {
        setIsLoading(false);
      }
    }
  }, [notifica, isMounted]);

  React.useEffect(() => {
    if (empleadoId) {
      loadData(empleadoId);
    } else {
      setRows([]);
    }
  }, [empleadoId, loadData]);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleadoId(empleado.id);
    },
    []
  );

  const handleNuevaConfiguracion = React.useCallback(() => {
    const nuevaFecha = dayjs().startOf('day');

    if (!nuevaFecha.isSame(fechaConfiguracion)) {
      setFechaConfiguracion(nuevaFecha);
      setRows([]);
    }
  }, [fechaConfiguracion]);

  const handleDuplicarConfiguracion = React.useCallback(async () => {
    if (!empleadoId) {
      notifica.show('Debe seleccionar un empleado para duplicar su configuración',
        { severity: 'warning', autoHideDuration: 5000 });
      return;
    }

    setIsLoading(true);
    try {
      const nuevaFechaCreacion = dayjs()

      const nuevosHorarios = await api().usuarios.duplicarHorario(empleadoId, nuevaFechaCreacion);
      if (isMounted.current) {
        setRows(nuevosHorarios);
        setFechaConfiguracion(nuevaFechaCreacion);
      }

      notifica.show('Configuración duplicada correctamente',
        { severity: 'success', autoHideDuration: 5000 });
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('horario-configurador.duplicar', error);
        notifica.show('Error al duplicar la configuración', { severity: 'error' });
      }
    } finally {
      if (isMounted.current) {
        setIsLoading(false);
      }
    }
  }, [empleadoId, fechaConfiguracion, notifica]);

  const handleNuevoHorario = React.useCallback(() => {
    if (!empleadoId) {
      notifica.show('Debe seleccionar un empleado', { severity: 'warning', autoHideDuration: 5000 });
      return;
    }

    navigate('/horarios/nuevo', {
      state: {
        usuarioId: empleadoId,
        fechaCreacion: fechaConfiguracion
      }
    });
  }, [navigate, empleadoId, fechaConfiguracion, notifica]);

  const handleEditarHorario = React.useCallback((id: GridRowId) => () => {
    navigate(`/horarios/${id}`);
  }, [navigate]);

  const handleEliminarHorario = React.useCallback((id: GridRowId) => async () => {
    const ok = await confirm('¿Está seguro de que desea eliminar el horario?', {
      title: 'Eliminar horario',
      okText: 'ELIMINAR',
      severity: 'warning'
    });

    if (ok) {
      try {
        await api().usuarios.eliminarHorario(Number(id));
        notifica.show('Horario eliminado correctamente',
          { severity: 'success', autoHideDuration: 5000 });
        if (isMounted.current) {
          if (empleadoId) loadData(empleadoId);
        }
      } catch (error) {
        if (error instanceof NetErrorControlado) return;
        logError('horario-configurador.eliminar', error);
        notifica.show('Error al eliminar el horario', { severity: 'error', autoHideDuration: 5000 });
      }
    }
  }, [confirm, api, notifica, empleadoId, loadData]);

  const columns = React.useMemo<GridColDef[]>(
    () => [
      {
        field: 'dia',
        headerName: 'DÍA',
        flex: 1,
        valueGetter: (_, row: ConfigHorario) =>
          diaSemanaToPalabra[row.horario.dia as DiaSemana],
      },
      {
        field: 'entrada',
        headerName: 'ENTRADA',
        flex: 1,
        valueGetter: (_, row: ConfigHorario) => row.horario.horaInicio,
      },
      {
        field: 'salida',
        headerName: 'SALIDA',
        flex: 1,
        valueGetter: (_, row: ConfigHorario) => row.horario.horaFin
      },
      {
        field: 'caducidadFechaIni',
        headerName: 'FECHA INICIO',
        flex: 1,
        valueGetter: (_, row: ConfigHorario) =>
          dateToStr(row.caducidadFechaIni as Dayjs) || 'Sin configurar',
        renderCell: (params) => (
          <span style={{
            color: params.value === 'Sin configurar' ? theme.palette.warning.main : 'inherit',
          }}>
            {params.value}
          </span>
        ),
      },
      {
        field: 'caducidadFechaFin',
        headerName: 'FECHA FIN',
        flex: 1,
        valueGetter: (_, row: ConfigHorario) =>
          dateToStr(row.caducidadFechaFin as Dayjs) || 'Sin configurar',
        renderCell: (params) => (
          <span style={{
            color: params.value === 'Sin configurar' ? theme.palette.warning.main : 'inherit',
          }}>
            {params.value}
          </span>
        ),
      },
      {
        field: 'actions',
        type: 'actions',
        flex: 1,
        align: 'right',
        getActions: ({ id }) => [
          <Tooltip title="Modificar" key="modificar-tooltip">
            <GridActionsCellItem
              icon={<EditIcon />}
              label="Editar"
              onClick={handleEditarHorario(id)}
            />
          </Tooltip>,
          <Tooltip title="Eliminar" key="eliminar-tooltip">
            <GridActionsCellItem
              icon={<DeleteIcon />}
              label="Eliminar"
              onClick={handleEliminarHorario(id)}
            />
          </Tooltip>,
        ],
      },
    ],
    [handleEditarHorario, handleEliminarHorario, theme]
  );


  const columnGroupingModel: GridColumnGroupingModel =
    React.useMemo(() => [
      {
        groupId: 'caducidad',
        headerName: 'CADUCIDAD',
        children: [
          { field: 'caducidadFechaIni' }, { field: 'caducidadFechaFin' }],
      },
    ], []);

  return (
    <PageContainer title="Configuración de Horarios">
      <Box sx={{ ...FULL_HEIGHT_WIDTH, display: 'flex', flexDirection: 'column', p: 2 }}>
        {/* Cabecera */}
        <Grid container spacing={2} alignItems="center">
          <Grid size={{ xs: 12, md: 6 }}>
            <TextField
              label="Última fecha de configuración"
              value={dateToStr(fechaConfiguracion)}
              slotProps={{
                input: {
                  readOnly: true,
                },
              }}
              fullWidth
              variant="outlined"
            />
          </Grid>
          <Grid size={{ xs: 12, md: 6 }} sx={{ display: 'flex', justifyContent: { xs: 'flex-start', md: 'flex-end' } }}>
            <ButtonGroup variant="outlined" aria-label="acciones configuración">
              <Button onClick={handleNuevaConfiguracion}>NUEVA</Button>
              <Button onClick={handleDuplicarConfiguracion}>DUPLICAR</Button>
            </ButtonGroup>
          </Grid>
        </Grid>

        <Divider sx={{ my: 2, width: '100%' }} />

        <Grid container spacing={2} alignItems="center" sx={{ mb: 2 }}>
          <Grid size={{ xs: 12, md: 6 }}>
            <SelectorEmpleado
              onChange={handleEmpleadoChange}
              disabled={isLoading}
              onLoadingChange={setIsLoading}
              usuarioPorDefecto={empleadoId ?? undefined}
            />
          </Grid>
          <Grid size={{ xs: 12, md: 6 }} sx={{ display: 'flex', justifyContent: { xs: 'flex-start', md: 'flex-end' } }}>
            <Button variant="contained" startIcon={<AddIcon />} sx={{ mt: 3 }} onClick={handleNuevoHorario}>
              NUEVO HORARIO
            </Button>
          </Grid>
        </Grid>

        <Box sx={{ flex: 1 }}>
          <DataGrid
            rows={rows}
            columns={columns}
            columnGroupingModel={columnGroupingModel}
            disableRowSelectionOnClick
            initialState={{
              pagination: {
                paginationModel: { pageSize: 25, page: 0 },
              },
            }}
            loading={isLoading}
            showToolbar
            slotProps={{
              loadingOverlay: {
                variant: 'circular-progress',
                noRowsVariant: 'circular-progress',
              },
              baseIconButton: {
                size: 'small',
              },
            }}
          />
        </Box>
      </Box>
    </PageContainer>
  );
}