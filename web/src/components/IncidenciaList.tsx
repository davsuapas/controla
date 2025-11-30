import * as React from 'react';
import {
  GridColDef,
  GridColumnGroupingModel,
  GridActionsCellItem,
  GridRowParams,
  DataGrid,
} from '@mui/x-data-grid';
import { api } from '../api/fabrica';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { logError } from '../error';
import { EstadoIncidencia, Incidencia, NombresEstadoIncidencia, NombresTipoIncidencia, TipoIncidencia } from '../modelos/incidencias';
import { Button, Card, CardContent, FormControl, Grid, InputLabel, MenuItem, Popover, Select, Stack, Tooltip, useTheme, Typography, Box, Chip, Divider, useMediaQuery } from '@mui/material';
import dayjs from 'dayjs';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';
import { dataGridStyles } from '../theme/customizations/dataGrid';
import { timeToStr } from '../modelos/formatos';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { SelectorFechas, SelectorFechasRef } from './SelectorFechas';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';

// Función para obtener el color del estado
function getEstadoColor(estado: EstadoIncidencia):
  "default" | "primary" | "secondary" | "error" | "info" | "success" | "warning" {
  switch (estado) {
    case EstadoIncidencia.Resuelta:
      return 'success';
    case EstadoIncidencia.ErrorResolver:
      return 'error';
    case EstadoIncidencia.Conflicto:
    case EstadoIncidencia.Rechazada:
      return 'warning';
    case EstadoIncidencia.Solicitud:
      return 'info';
    default:
      return 'default';
  }
};

// Extiende a incidencias con campos de control
export class IncidenciaGrid extends Incidencia {
  accion?: EstadoIncidencia;
  errorServidor?: boolean;

  static fromIncidencia(inc: Incidencia): IncidenciaGrid {
    const incGrid = new IncidenciaGrid(inc);
    return incGrid;
  }
}

// Define el tipo para las acciones
export interface IncidenciaAction {
  icon: React.ReactElement;
  label: string;
  tooltip?: string;
  onClick: (row: IncidenciaGrid) => void;
  show?: (row: IncidenciaGrid) => boolean;
}

interface IncidenciaListProps {
  estadosFiltro: EstadoIncidencia[];
  usuarioFiltro?: number;
  actions?: IncidenciaAction[];
  columnaAccion?: boolean;
  rows?: IncidenciaGrid[];
  setRows?: React.Dispatch<React.SetStateAction<IncidenciaGrid[]>>;
  isLoading?: boolean;
  setIsLoading?: React.Dispatch<React.SetStateAction<boolean>>;
  onFilterChange?: (filters: {
    fechaInicio: dayjs.Dayjs | null;
    fechaFin: dayjs.Dayjs | null;
    estados: EstadoIncidencia[];
    supervisor: boolean;
    usuarioId: number | null;
  }) => void;
}

// Gestiona la lista de incidencias con la que un gestor o
// un empleado puede trabajar. Dependiendo del rol del 
// usuario se pueden hacer una o varias accciones y filtrar
// por unos determinados estados
export default function IncidenciaList(props: IncidenciaListProps) {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const [isExpanded, setIsExpanded] = React.useState<boolean>(false);

  const selectorFechasRef = React.useRef<SelectorFechasRef>(null);

  const [estado, setEstado] = React.useState<number>(0);

  const notifica = useNotifications();
  const usuario = useUsuarioLogeado().getUsrLogeado();
  const supervisor = usuario.tieneRol(RolID.Supervisor);

  // Usa las rows externas si se proporcionan, si no usa el estado interno
  const [internalRows, setInternalRows] = React.useState<IncidenciaGrid[]>([]);
  const [internalIsLoading, setInternalIsLoading] = React.useState(true);

  // Propiedades para mostrar el detalle de las incidencias
  const [popoverOpen, setPopoverOpen] = React.useState(false);
  const [popoverAnchorEl, setPopoverAnchorEl] = React.useState<HTMLElement | null>(null);
  const [selectedIncidencia, setSelectedIncidencia] = React.useState<IncidenciaGrid | null>(null);

  // Conecta loading al estado interno o externo
  const isLoading = props.isLoading !== undefined ? props.isLoading : internalIsLoading;
  const setIsLoading = props.setIsLoading !== undefined ? props.setIsLoading : setInternalIsLoading;

  // Conecta las rows al estado interno o externo
  // Las filas se pueden pasar desde el componente padre
  // de esta forma se puede controlar la recarga desde fuera
  const rowsState = props.rows !== undefined ? props.rows : internalRows;
  const setRowsState = props.setRows !== undefined ?
    props.setRows : setInternalRows;

  // Carga las incidencias dependiendo de los filtros
  // Si en envía el usuario se filtran las solicitudes
  // para ese usuario
  const loadData = React.useCallback(async () => {
    setIsLoading(true);

    let listData: Incidencia[] = [];
    try {
      // Si no se selecciona un estado para filtrar
      // se filtra por todos
      const estadosFiltrar = estado == 0
        ? props.estadosFiltro
        : [estado];

      if (!selectorFechasRef.current) {
        console.log('filtrosRef.current es vacío en IncidenciaList');
        return;
      }

      const { fechaInicio, fechaFin } = selectorFechasRef.current.getFormData();

      // Notificar al padre sobre los filtros aplicados
      if (props.onFilterChange) {
        props.onFilterChange({
          fechaInicio,
          fechaFin,
          estados: estadosFiltrar,
          supervisor,
          usuarioId: props.usuarioFiltro ? props.usuarioFiltro : null
        });
      }

      listData = await api().inc.incidencias(
        fechaInicio,
        fechaFin,
        estadosFiltrar,
        supervisor,
        props.usuarioFiltro ? props.usuarioFiltro : null)
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('incidencias-listar.cargar', error);
        notifica.show(
          'Error inesperado al cargar la lista de incidencias',
          {
            severity: 'error',
            autoHideDuration: 5000,
          },
        );
      }
    }

    // Convierte Incidencia[] a IncidenciaGrid[]
    const gridData: IncidenciaGrid[] = listData.map(incidencia => (
      IncidenciaGrid.fromIncidencia(incidencia)
    ));
    setRowsState(gridData);

    setIsLoading(false);
  }, [selectorFechasRef]);

  React.useEffect(() => {
    loadData();
  }, []);

  const toggleExpand = React.useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  const columns = React.useMemo<GridColDef[]>(
    () => {
      const baseColumns: GridColDef[] = [
        {
          field: 'id',
          headerName: 'ID',
          width: 10
        },
        {
          field: 'estado',
          headerName: 'ESTADO',
          minWidth: 140,
          valueGetter: (estado) =>
            NombresEstadoIncidencia[estado as EstadoIncidencia],
          renderCell: (params) => {
            return (
              <Chip
                label={
                  NombresEstadoIncidencia[params.row.estado as EstadoIncidencia]
                }
                color={getEstadoColor(params.row.estado)}
                size="small"
                variant="outlined"
              />
            );
          }
        },
        {
          field: 'fechaSolicitud',
          headerName: 'FECHA',
          minWidth: 120,
        },
        {
          field: 'tipo',
          headerName: 'TIPO',
          minWidth: 170,
          renderCell: (params) => {
            return (
              <Chip
                label={NombresTipoIncidencia[params.row.tipo as TipoIncidencia]}
                size="small"
                color="primary"
                variant="outlined"
              />
            );
          }
        },
        {
          field: 'marcaje',
          headerName: 'E/S',
          minWidth: 140,
          renderCell: (params) => {
            return timeToStr(params.row.marcaje?.hora_inicio) +
              ' - ' + timeToStr(params.row.marcaje?.hora_fin)
          }
        },
        {
          field: 'fecha',
          headerName: 'FECHA',
          minWidth: 120,
        },
        {
          field: 'rectificacion',
          headerName: 'E/S',
          minWidth: 140,
          renderCell: (params) => {
            return timeToStr(params.row.horaInicio) +
              ' - ' + timeToStr(params.row.horaFin)
          }
        },
      ];

      // Solo agregar columna de acciones si se pasaron acciones
      if (props.actions && props.actions.length > 0) {
        if (props.columnaAccion) {
          baseColumns.push({
            field: 'accion',
            headerName: 'ACCION',
            minWidth: 120,
            renderCell: (params) => {
              const nombre = NombresEstadoIncidencia[
                params.value as EstadoIncidencia] ?? '';
              return (
                <span style={{
                  color: theme.palette.success.main, fontWeight: 600
                }}>
                  {nombre}
                </span>
              );
            }
          })
        }

        baseColumns.push({
          field: 'actions',
          type: 'actions',
          flex: 1,
          align: 'right',
          getActions: ({ row }: GridRowParams<Incidencia>) => {
            return props.actions!
              .filter(action => !action.show || action.show(row))
              .map((action, index) => {
                const actionItem = (
                  <GridActionsCellItem
                    key={`action-${index}`}
                    icon={action.icon}
                    label={action.label}
                    onClick={() => action.onClick(row)}
                  />
                );

                return action.tooltip ? (
                  <Tooltip title={action.tooltip} key={`tooltip-${index}`}>
                    {actionItem}
                  </Tooltip>
                ) : actionItem;
              });
          },
        });
      }

      return baseColumns;
    },
    [props.actions],
  );

  const columnGroupingModel: GridColumnGroupingModel = [
    {
      groupId: 'solicitud',
      headerName: 'SOLICITUD',
      children: [
        { field: 'fechaSolicitud' }, { field: 'tipo' }],
    },
    {
      groupId: 'rectificacion',
      headerName: 'RECTIFICACIÓN MARCAJE',
      description: 'Dependen del tipo de solicitud',
      children: [
        { field: 'fecha' }, { field: 'rectificacion' }],
    },
    {
      groupId: 'marcaje',
      headerName: 'MARCAJE',
      description: 'Referencia al marcaje realizado',
      children: [
        { field: 'marcaje' }],
    },
  ];

  // Filtra la tabla de incidencias
  const handleFiltrar = () => {
    loadData();
  };

  // Cuando se realiza click sobre la fila se muestra el detalle
  const handleRowClick = (params: GridRowParams<IncidenciaGrid>, event: React.MouseEvent<HTMLElement>) => {
    setSelectedIncidencia(params.row);
    setPopoverAnchorEl(event.currentTarget);
    setPopoverOpen(true);
  };

  // Cuando se cieerra el pospover se inicializa el estado
  const handlePopoverClose = () => {
    setPopoverOpen(false);
    setPopoverAnchorEl(null);
    setSelectedIncidencia(null);
  };

  return (
    <Stack spacing={2} sx={{ height: '100%', mt: 1.5 }}>
      <Grid container spacing={2}
        sx={{
          ml: 0.2, mb: 2, width: '100%',
          display: isMobile && isExpanded ? 'none' : 'flex',
          flexShrink: 0
        }}>
        <SelectorFechas
          ref={selectorFechasRef}
          labelUltimosRegistros={'Últimas incidencias'} />
        <Grid size={{ xs: 12, sm: 12, md: 2 }}>
          <FormControl>
            <InputLabel>Estado</InputLabel>
            <Select
              name="estado"
              value={estado}
              onChange={value => setEstado(value.target.value)}
              autoWidth
            >
              <MenuItem value="0">
                Todos
              </MenuItem>
              {props.estadosFiltro.map(estado => (
                <MenuItem key={estado} value={estado}>
                  {NombresEstadoIncidencia[estado]}
                </MenuItem>
              ))}
            </Select>              </FormControl>
        </Grid>
        <Grid size={{ xs: 12, sm: 12, md: 1 }}>
          <Button
            variant="contained"
            sx={{
              width: { xs: '100%', sm: 'auto' },
              minWidth: 120,
              mt: 0.5
            }}
            disabled={isLoading}
            onClick={handleFiltrar}
          >
            FILTRAR
          </Button>
        </Grid>
      </Grid>
      {/* Botón para expandir/contraer en móviles */}
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
      <DataGrid
        rows={rowsState}
        columnGroupingModel={columnGroupingModel}
        columns={columns}
        ignoreDiacritics
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
          params.row.errorServidor ? 'fila-con-error' : ''
        }
        sx={{
          '& .fila-con-error': dataGridStyles.marcarFila(theme)
        }}
        slotProps={{
          loadingOverlay: {
            variant: 'circular-progress',
            noRowsVariant: 'circular-progress',
          },
          baseIconButton: {
            size: 'small',
          },
          row: {
            onMouseEnter: (event: React.MouseEvent<HTMLDivElement>) => {
              const rowElement = event.currentTarget;
              const rowId = rowElement.getAttribute('data-id');
              const row = rowsState.find(r => r.id.toString() === rowId);

              if (row?.errorServidor) {
                rowElement.title = 'No se ha podido procesar la incidencia por un error fatal. Inténtelo de nuevo.';
              }
            },
          } as any,
        }} onRowClick={handleRowClick}
      />
      {selectedIncidencia && (
        <IncidenciaDetalle
          incidencia={selectedIncidencia}
          open={popoverOpen}
          anchorEl={popoverAnchorEl}
          onClose={handlePopoverClose}
        />
      )}
    </Stack>
  );
}

// Muestra el detalle de una incidencia en un popover
interface IncidenciaDetalleProps {
  incidencia: Incidencia;
  open: boolean;
  anchorEl: HTMLElement | null;
  onClose: () => void;
}

function IncidenciaDetalle({
  incidencia,
  open,
  anchorEl,
  onClose
}: IncidenciaDetalleProps) {
  return (
    <Popover
      open={open}
      anchorEl={anchorEl}
      onClose={onClose}
      anchorOrigin={{
        vertical: 'center',
        horizontal: 'center',
      }}
      transformOrigin={{
        vertical: 'center',
        horizontal: 'center',
      }}
      sx={{
        '& .MuiPopover-paper': {
          maxWidth: 400,
          minWidth: 300,
        }
      }}
    >
      <Card variant="outlined">
        <CardContent sx={{ p: 2 }}>
          {/* Header */}
          <Box sx={{ mb: 2 }}>
            <Typography variant="h6" component="h2" sx={{ mb: 1, fontWeight: 600 }}>
              Incidencia ({incidencia.fechaSolicitud.toString()})
            </Typography>
            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', alignItems: 'center' }}>
              <Chip
                label={NombresEstadoIncidencia[incidencia.estado]}
                color={getEstadoColor(incidencia.estado)}
                size="small"
              />
              <Chip
                label={NombresTipoIncidencia[incidencia.tipo]}
                size="small"
                color="primary"
                variant="outlined"
              />
            </Box>
          </Box>

          {/* Fecha resolución */}
          {incidencia.fechaResolucion && (
            <Box sx={{ mb: 2 }}>
              <Typography variant="body2">
                <strong>Fecha resolución:</strong> {incidencia.fechaResolucion.toString()}
              </Typography>
            </Box>
          )}

          {/* Fecha estado */}
          {incidencia.fechaEstado && (
            <Box sx={{ mb: 2 }}>
              <Typography variant="body2">
                Fecha estado: {incidencia.fechaEstado.toString()}
              </Typography>
            </Box>
          )}

          {/* Error */}
          {incidencia.error && (
            <>
              <Divider sx={{ my: 2 }} />
              <Box>
                <Typography variant="body2" fontWeight="medium" sx={{ mb: 0.5, color: 'text.secondary' }}>
                  Error
                </Typography>
                <Typography variant="body2" sx={{ fontStyle: 'italic', color: 'warning.main' }}>
                  {incidencia.error}
                </Typography>
              </Box>
            </>
          )}

          <Divider sx={{ my: 2 }} />

          {/* Datos rectificación */}
          {incidencia.tipo !== TipoIncidencia.EliminacionMarcaje && (
            <Box sx={{ mb: 2 }}>
              <Typography variant="body2" fontWeight="medium" sx={{ mb: 1, color: 'text.secondary' }}>
                Datos rectificación
              </Typography>
              <Typography variant="body2">
                <strong>Empleado:</strong> {(incidencia.usuario as DescriptorUsuario).
                  nombreCompleto()}
              </Typography>
              <Typography variant="body2">
                <strong>Fecha:</strong> {incidencia.fecha.toString()} | {incidencia.horaInicio || '--:--'} {incidencia.horaFin || '--:--'}
              </Typography>
            </Box>
          )}

          {/* Referencia marcaje */}
          {incidencia.marcaje && (
            <Box sx={{ mb: 2 }}>
              <Typography variant="body2" fontWeight="medium" sx={{ mb: 1, color: 'text.secondary' }}>
                Referencia marcaje
              </Typography>
              <Typography variant="body2">
                <strong>ID:</strong> {incidencia.marcaje.id} | {incidencia.marcaje.hora_inicio} - {incidencia.marcaje.hora_fin || '--:--'}
              </Typography>
            </Box>
          )}

          <Divider sx={{ my: 2 }} />

          {/* Usuarios */}
          <Box sx={{ mb: 2 }}>
            <Typography variant="body2" fontWeight="medium" sx={{ mb: 0.5, color: 'text.secondary' }}>
              Creado por
            </Typography>
            <Typography variant="body2" sx={{ mb: 1 }}>
              {(incidencia.usuarioCreador as DescriptorUsuario).nombreCompleto()}
            </Typography>

            {incidencia.usuarioGestor && (
              <>
                <Typography variant="body2" fontWeight="medium" sx={{ mb: 0.5, color: 'text.secondary' }}>
                  Gestionado por
                </Typography>
                <Typography variant="body2">
                  {(incidencia.usuarioGestor as DescriptorUsuario).nombreCompleto()}
                </Typography>
              </>
            )}
          </Box>

          {((incidencia.motivoSolicitud || incidencia.motivoRechazo) && (
            <Divider sx={{ my: 2 }} />
          ))}

          {/* Motivos */}
          <Box sx={{ mb: 2 }}>
            {incidencia.motivoSolicitud && (
              <Box sx={{ mb: 2 }}>
                <Typography variant="body2" fontWeight="medium" sx={{ mb: 0.5, color: 'text.secondary' }}>
                  Motivo solicitud
                </Typography>
                <Typography variant="body2" sx={{ fontStyle: 'italic' }}>
                  {incidencia.motivoSolicitud}
                </Typography>
              </Box>
            )}

            {incidencia.motivoRechazo && (
              <Box sx={{ mb: 2 }}>
                <Typography variant="body2" fontWeight="medium" sx={{ mb: 0.5, color: 'text.secondary' }}>
                  Motivo rechazo
                </Typography>
                <Typography variant="body2" sx={{ fontStyle: 'italic' }}>
                  {incidencia.motivoRechazo}
                </Typography>
              </Box>
            )}
          </Box>
        </CardContent>
      </Card>
    </Popover>
  );
};