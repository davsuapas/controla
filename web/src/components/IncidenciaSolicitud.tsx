import PageContainer from './PageContainer';
import * as React from 'react';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import Stack from '@mui/material/Stack';
import {
  DataGrid,
  GridActionsCellItem,
  GridColDef,
} from '@mui/x-data-grid';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { api } from '../api/fabrica';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { logError, validarFechaHora } from '../error';
import { DescriptorMarcaje, Marcaje } from '../modelos/marcaje';
import dayjs, { Dayjs } from 'dayjs';
import LocalizationProviderES from '../theme/location';
import { useTheme } from '@mui/material/styles';
import AutoFixHighIcon from '@mui/icons-material/AutoFixHigh';
import PlaylistRemoveIcon from '@mui/icons-material/PlaylistRemove';
import Tooltip from '@mui/material/Tooltip';
import Button from '@mui/material/Button';
import Grid from '@mui/material/Grid';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import Box from '@mui/material/Box';
import DialogActions from '@mui/material/DialogActions';
import { createDayjsFromTime, dateToStr } from '../modelos/formatos';
import { TimePicker } from '@mui/x-date-pickers/TimePicker';
import TextField from '@mui/material/TextField';
import { Incidencia, TipoIncidencia } from '../modelos/incidencias';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { dataGridStyles } from '../theme/customizations/dataGrid';
import SelectorEmpleado from './SelectorEmpleado';
import { DescriptorUsuario, filtroUsuarioRegistra, RolID } from '../modelos/usuarios';
import { useIsMounted } from '../hooks/useComponentMounted';

const HORA_NO_VALIDA = 'Hora no valida';

// Componente que expone los marcajes por fecha y permite
// realiza solicitud de incidencias para un usuario.
// Las incidencias pueden ser: Salidas erróneas,
// eliminación de algún marcaje (solo roles específicos)
// y creación de uno nueva.
// Si el usuario tiene el rol registrador o supervisor podrá
// crear incidencias de los usuarios que registro en nombre
// del empleado.
// El supervisor puede crear incidencias de todos marcajes
// realizados por los usuarios registradores
// Esta pantalla solo puede ser usada por empleados,
// registradores o supervisores
export default function SolicitudIncidencia() {
  const theme = useTheme();
  const notifica = useNotifications();
  const usuarioLog = useUsuarioLogeado().getUsrLogeado()
  const isMounted = useIsMounted();

  const [rows, setRows] = React.useState<Marcaje[]>([]);
  const [isLoading, setIsLoading] = React.useState(true);
  const [fecha, setFecha] = React.useState<Dayjs | null>(dayjs());

  const [solicitudesProcesadas, setSolicitudesProcesadas] =
    React.useState<Set<number>>(new Set());

  const [empleado, setEmpleado] =
    React.useState<number | undefined>(usuarioLog.id);

  // Estados para el modal con la información de la solictud
  const [modalOpenInfo, setModalOpenInfo] = React.useState(false);
  const [tipoSolicitud, setTipoSolicitud] =
    React.useState<TipoIncidencia | undefined>(undefined);
  const [marcajeSeleccionado, setMarcajeSeleccionado] =
    React.useState<Marcaje | undefined>(undefined);

  const solicitudEliminacion =
    usuarioLog.tieneRol(RolID.Registrador) || usuarioLog.tieneRol(RolID.Supervisor)

  const usuarioSoloEmpleado = usuarioLog.tieneRol(RolID.Empleado) &&
    !usuarioLog.tieneRol(RolID.Registrador) && !usuarioLog.tieneRol(RolID.Supervisor);

  const handleEmpleadoChange = React.useCallback(
    (empleado: DescriptorUsuario) => {
      setEmpleado(empleado.id);
    },
    []
  );

  // Almacena todas las solictudes creadas para 
  // que puedan ser consultadas, por ejemplo para
  // ser marcadas en el grid
  const agregarSolicitudCreada = React.useCallback((marcajeId: number) => {
    setSolicitudesProcesadas(prev => new Set(prev).add(marcajeId));
  }, []);

  const limpiarSolicitudCreada = React.useCallback(() => {
    setSolicitudesProcesadas(new Set());
  }, []);

  // Carga los marcajes por fecha
  const loadData = React.useCallback(
    async (
      usuarioId: number,
      fecha: Dayjs) => {
      setIsLoading(true);

      let listData: Marcaje[] = [];

      try {
        listData = await api().marcajes.marcajesSinInc(
          usuarioId.toString(),
          fecha,
          filtroUsuarioRegistra(usuarioId, usuarioLog)?.toString()
        );
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('solicitud-incidencia.cargar', error);
          notifica.show('Error inesperado al cargar la lista de marcajes', {
            severity: 'error',
            autoHideDuration: 5000,
          });
        }
      }

      if (isMounted.current) {
        setRows(listData);
        limpiarSolicitudCreada();
        setIsLoading(false);
      };
    },
    [usuarioLog.id, usuarioLog.roles, limpiarSolicitudCreada,
      notifica, setRows, setIsLoading]
  );

  React.useEffect(() => {
    if (usuarioSoloEmpleado && fecha) {
      loadData(usuarioLog.id, fecha);
      return;
    }

    if (empleado && fecha) {
      loadData(empleado, fecha);
    }
  }, [empleado, fecha, loadData, usuarioSoloEmpleado, usuarioLog.id]);

  // Permite abrir un formalario para corregir marcajes
  // mediante solicitud
  const abrirModalInfo = React.useCallback(
    (tipo: TipoIncidencia, marcaje?: Marcaje) => {
      setMarcajeSeleccionado(marcaje);
      setTipoSolicitud(tipo);
      setModalOpenInfo(true);
    },
    []
  );

  // Procesa las solicitud con las correciones
  const procesarSolicitud = React.useCallback(
    async (
      tipo: TipoIncidencia,
      info: InfoSolicitud,
      marcaje?: Marcaje
    ) => {
      let msgNotifica = 'Solicitud no reconocida';

      if (!fecha) {
        return;
      }

      switch (tipo) {
        case TipoIncidencia.CorrecionSalida:
          msgNotifica = 'Solicitud "salida errónea" creada satisfactoriamente';
          break;

        case TipoIncidencia.EliminacionMarcaje:
          msgNotifica = 'Solicitud de eliminación creada satisfactoriamente';
          break;

        case TipoIncidencia.NuevoMarcaje:
          msgNotifica = 'Solicitud "nuevo marcaje" creada satisfactoriamente';
          break;

        default:
          console.warn('Tipo de solicitud no reconocido:', tipo);
          notifica.show(msgNotifica, {
            severity: 'success',
            autoHideDuration: 5000,
          });
          return;
      }

      try {
        await api().inc.crearIncidencia(
          Incidencia.crearSolicitud(
            tipo,
            empleado!,
            fecha,
            info.horaEntrada ?? null,
            info.horaSalida ?? null,
            marcaje ? new DescriptorMarcaje(marcaje.id, null, null) : null,
            usuarioLog.id,
            info.motivo ?? null
          )
        );

        notifica.show(msgNotifica, {
          severity: 'success',
          autoHideDuration: 5000,
        });

        if (marcaje) {
          if (isMounted.current) {
            agregarSolicitudCreada(marcaje.id);
          };
        }
      } catch (error) {
        if (error instanceof NetErrorControlado) {
          return;
        }

        logError('solicitud-incidencia.crear', error);

        notifica.show('Error inesperado al crear una solicitud de incidencia', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    },
    []
  );

  // Cierra la modal mediante un borón aceptar y otro cancelar
  const cerrarModalInfo = React.useCallback(
    (info: InfoSolicitud | undefined) => {
      setModalOpenInfo(false);

      if (info) {
        procesarSolicitud(tipoSolicitud!, info, marcajeSeleccionado);
      }

      setTipoSolicitud(undefined);
      setMarcajeSeleccionado(undefined);
    },
    [tipoSolicitud, marcajeSeleccionado, procesarSolicitud]);

  // Abre una solictud para corregir salidas erróneas
  const handleSolicitudClick = React.useCallback(
    (marcaje: Marcaje) => () => {
      abrirModalInfo(TipoIncidencia.CorrecionSalida, marcaje);
    },
    [fecha, empleado, agregarSolicitudCreada, usuarioLog.id, notifica]
  );

  // Abre una solictud de eliminación
  const handleEliminarClick = React.useCallback(
    (marcaje: Marcaje) => () => {
      abrirModalInfo(TipoIncidencia.EliminacionMarcaje, marcaje);
    },
    []
  );

  // Abre una solictud para crear un nuevo marcaje
  const handleMarcajeNoRealizado = () => {
    abrirModalInfo(TipoIncidencia.NuevoMarcaje);
  };

  const columns = React.useMemo<GridColDef[]>(
    () => [
      {
        field: 'horaInicio',
        headerName: 'ENTRADA',
        flex: 1,
        minWidth: 100,
      },
      {
        field: 'horaFin',
        headerName: 'SALIDA',
        flex: 1,
        minWidth: 100,
        renderCell: (params) => {
          return (
            <span
              style={{
                color: params.row.horaFin ? undefined : theme.palette.error.main,
                fontWeight: params.row.horaFin ? 'normal' : 'bold',
              }}
            >
              {params.row.horaFinToStr()}
            </span>
          );
        },
      },
      {
        field: 'horario_horaInicio',
        headerName: 'HORA A ENTRAR',
        flex: 1,
        minWidth: 150,
        valueGetter: (_, row) => row.horario.horaInicio,
      },
      {
        field: 'horario_horaFin',
        headerName: 'HORA A SALIR',
        flex: 1,
        minWidth: 150,
        valueGetter: (_, row) => row.horario.horaFin,
      },
      {
        field: 'usuarioReg',
        headerName: 'REGISTRADOR EXTERNO',
        flex: 2,
        minWidth: 200,
        valueGetter: (_, row) =>
          (row.usuario_reg ? row.usuario_reg.nombreCompleto() : ''),
      },
      {
        field: 'actions',
        type: 'actions',
        flex: 1,
        minWidth: 100,
        align: 'right',
        getActions: ({ row }) => {
          if (solicitudesProcesadas.has(row.id)) {
            return [];
          }
          return [
            <Tooltip title="Corregir salida" key="salida-erronea-tooltip">
              <GridActionsCellItem
                key="solicitud-salida-erronea"
                icon={<AutoFixHighIcon />}
                label="Corregir salida"
                onClick={handleSolicitudClick(row)}
              />
            </Tooltip>,
            solicitudEliminacion && (
              <Tooltip title="Eliminar marcaje" key="elimi-marcaje-tooltip">
                <GridActionsCellItem
                  key="eliminacion-marcaje"
                  icon={<PlaylistRemoveIcon />}
                  label="Eliminar marcaje"
                  onClick={handleEliminarClick(row)}
                />
              </Tooltip>
            ),
          ].filter(Boolean);
        },
      },
    ],
    [solicitudesProcesadas, theme, solicitudEliminacion,
      handleSolicitudClick, handleEliminarClick]
  );

  return (
    <PageContainer title={'Solicitud incidencia'}>
      <Box sx={{ mt: 3, ...FULL_HEIGHT_WIDTH }}>
        <LocalizationProviderES>
          <Stack spacing={2} sx={{ height: '100%' }}>
            {!usuarioSoloEmpleado && (
              <>
                <SelectorEmpleado
                  onChange={handleEmpleadoChange}
                  disabled={false}
                  onLoadingChange={setIsLoading}
                  usuarioPorDefecto={usuarioLog.id}
                />
                <Box sx={{ mb: 2 }} />
              </>
            )}
            <DatePicker
              name="fecha"
              label="Fecha"
              value={fecha}
              onChange={(v) => setFecha(v)}
              sx={{ width: '100%' }}
            />
            <Grid container spacing={1} justifyContent="flex-end">
              <Grid size={{ xs: 12, sm: 12, md: 5 }}
                sx={{ display: 'flex', flexDirection: 'column' }}>
                <Button
                  variant="contained"
                  sx={{
                    width: { xs: '100%', sm: 'auto' },
                    minWidth: 120,
                  }}
                  onClick={handleMarcajeNoRealizado}
                >
                  NUEVO MARCAJE
                </Button>
              </Grid>
            </Grid>
            <DataGrid
              rows={rows}
              columns={columns}
              ignoreDiacritics
              disableColumnSorting
              disableColumnFilter
              disableRowSelectionOnClick
              pageSizeOptions={[]}
              initialState={{
                pagination: {
                  paginationModel: { pageSize: 25, page: 0 },
                },
              }}
              loading={isLoading}
              showToolbar
              getRowClassName={(params) =>
                solicitudesProcesadas.has(params.row.id) ? 'fila-con-solicitud' : ''
              }
              sx={{
                '& .fila-con-solicitud': dataGridStyles.marcarFila(theme)
              }}
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
            {fecha && tipoSolicitud && (
              <ModalInfoSolicitud
                open={modalOpenInfo}
                onClose={cerrarModalInfo}
                fecha={fecha}
                {...crearModalInfoSolicitudProps(
                  fecha, tipoSolicitud,
                  undefined,
                  marcajeSeleccionado?.horaFin ?
                    marcajeSeleccionado.horaFin : undefined)!}
              />
            )}
          </Stack>
        </LocalizationProviderES>
      </Box>
    </PageContainer>
  );
}

export interface InfoSolicitud {
  horaEntrada: Dayjs | undefined;
  horaSalida: Dayjs | undefined;
  motivo?: string;
}

interface InfoSolicitudErrors {
  horaEntrada: string;
  horaSalida: string;
}

// Configura las propiedades para la pantalla de información
// de una solicitud
export function crearModalInfoSolicitudProps(
  fecha: dayjs.Dayjs,
  tipo: TipoIncidencia,
  horaInicio?: string,
  horaFin?: string
) {
  if (tipo === TipoIncidencia.CorrecionSalida) {
    return {
      titulo: 'CORREGIR SALIDA',
      mostrarEntrada: false,
      mostrarSalida: true,
      info: {
        horaEntrada: undefined,
        horaSalida: horaFin ? createDayjsFromTime(fecha, horaFin) : undefined
      }
    };
  }

  if (tipo === TipoIncidencia.NuevoMarcaje) {
    return {
      titulo: 'NUEVO MARCAJE',
      mostrarEntrada: true,
      mostrarSalida: true,
      info: {
        horaEntrada: horaInicio ? createDayjsFromTime(fecha, horaInicio) : undefined,
        horaSalida: horaFin ? createDayjsFromTime(fecha, horaFin) : undefined
      }
    };
  }

  if (tipo === TipoIncidencia.EliminacionMarcaje) {
    return {
      titulo: 'ELIMINAR MARCAJE',
      mostrarEntrada: false,
      mostrarSalida: false,
      info: { horaEntrada: undefined, horaSalida: undefined }
    };
  }
}

export interface ModalInfoSolicitudProps {
  open: boolean;
  onClose: (datos: InfoSolicitud | undefined) => void;
  fecha: Dayjs;
  info: InfoSolicitud;
  mostrarEntrada?: boolean;
  mostrarSalida?: boolean;
  titulo?: string;
}

// Permite editar la información para crear una solictud de incidenia
// Dependiendo del tipo la información puede ser diferente
// pidiendo la hora de entrada, salida y un campo para motivar
// la solictud.
export function ModalInfoSolicitud({
  open,
  onClose,
  fecha,
  info,
  mostrarEntrada = true,
  mostrarSalida = true,
  titulo = 'Información solicitud',
}: ModalInfoSolicitudProps) {
  const [infoSolicitud, setInfoSolicitud] =
    React.useState<InfoSolicitud>(info);
  const [formErrors, setFormErrors] =
    React.useState<Partial<InfoSolicitudErrors>>({});

  const notifica = useNotifications();

  // Maneja el cambio de los campos de fecha y hora
  const handleHorasChange = React.useCallback(
    (name: string, value: dayjs.Dayjs | null) => {
      const valida = validarFechaHora(value)
      setFormErrors(prev => ({
        ...prev,
        [name]: valida ? undefined : HORA_NO_VALIDA
      }));

      setInfoSolicitud(prev => ({
        ...prev,
        [name]: value
      }));
    }, [validarFechaHora]);

  const handleSubmit = React.useCallback(() => {
    const validaEntrada =
      !mostrarEntrada || validarFechaHora(infoSolicitud.horaEntrada); // Usa mostrarEntrada, validarFechaHora, infoSolicitud
    const validaSalida =
      !mostrarSalida || validarFechaHora(infoSolicitud.horaSalida); // Usa mostrarSalida, validarFechaHora, infoSolicitud

    if (validaEntrada && validaSalida) {
      onClose(infoSolicitud); // Usa onClose, infoSolicitud
    } else {
      setFormErrors({ // Usa setFormErrors
        horaEntrada: validaEntrada ? undefined : HORA_NO_VALIDA,
        horaSalida: validaSalida ? undefined : HORA_NO_VALIDA
      })

      notifica.show( // Usa notifica
        'Imposible realizar la solicitud. Corriga los errores',
        {
          severity: 'warning',
          autoHideDuration: 5000,
        },
      );
    }
  }, [mostrarEntrada, mostrarSalida, validarFechaHora,
    infoSolicitud, onClose, notifica]);

  const handleCancel = React.useCallback(() => {
    onClose(undefined);
  }, [onClose]);

  return (
    <Dialog open={open} onClose={handleCancel} maxWidth="sm" fullWidth>
      <DialogTitle>
        {titulo} - {dateToStr(fecha)}
      </DialogTitle>
      <DialogContent>
        <LocalizationProviderES>
          <Box sx={{ pt: 3 }}>
            {mostrarEntrada && (
              <Grid container spacing={2} alignItems="center" sx={{ mb: 2 }}>
                <Grid size={{ xs: 12, sm: 8 }}>
                  <TimePicker
                    label="Hora de Entrada"
                    value={infoSolicitud.horaEntrada || null}
                    onChange={value => handleHorasChange('horaEntrada', value)}
                    slotProps={{
                      textField: {
                        fullWidth: true,
                        error: !!formErrors.horaEntrada,
                        helperText: formErrors.horaEntrada ?? ' '
                      }
                    }}
                  />
                </Grid>
              </Grid>
            )}
            {mostrarSalida && (
              <Grid container spacing={2} alignItems="center">
                <Grid size={{ xs: 12, sm: 8 }}>
                  <TimePicker
                    label="Hora de Salida"
                    value={infoSolicitud.horaSalida || null}
                    onChange={value => handleHorasChange('horaSalida', value)}
                    format="HH:mm"
                    slotProps={{
                      textField: {
                        fullWidth: true,
                        error: !!formErrors.horaSalida,
                        helperText: formErrors.horaSalida ?? ' '
                      },
                    }}
                  />
                </Grid>
              </Grid>
            )}
            <Grid size={{ xs: 12, sm: 8 }} sx={{ mt: 2 }}>
              <TextField
                value={infoSolicitud.motivo ?? ''}
                onChange={e => {
                  // Crear una copia del objeto y actualizar la propiedad
                  setInfoSolicitud({
                    ...infoSolicitud,
                    motivo: e.target.value
                  });
                }}
                name="motivo"
                label="Motivo"
                helperText='Indique una aclaración si es necesario (máx. 200 caracteres)'
                fullWidth
                slotProps={{
                  htmlInput: {
                    maxLength: 200
                  }
                }}
              />
            </Grid>
          </Box>
        </LocalizationProviderES>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleCancel}>CANCELAR</Button>
        <Button onClick={handleSubmit} variant="contained">
          SOLICITAR
        </Button>
      </DialogActions>
    </Dialog>
  );
}