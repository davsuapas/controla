import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import CheckIcon from '@mui/icons-material/Check';
import CloseIcon from '@mui/icons-material/Close';
import ReplayIcon from '@mui/icons-material/Replay';
import CancelIcon from '@mui/icons-material/Cancel';
import SyncIcon from '@mui/icons-material/Sync';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import IncidenciaList, { IncidenciaAction, IncidenciaGrid } from './IncidenciaList';
import { EstadoIncidencia } from '../modelos/incidencias';
import React from 'react';
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import { api } from '../api/fabrica';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { IncidenciaProcesoDTO } from '../modelos/dto';
import dayjs from 'dayjs';
import { NetErrorControlado } from '../net/interceptor';
import { logError } from '../error';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { useIsMounted } from '../hooks/useComponentMounted';

// Pemite revisar las incidencias que se solicitan
// para rectificar los marcajes
export default function RevisionIncidencia() {
  const usuario = useUsuarioLogeado().getUsrLogeado();
  const dialogo = useDialogs();
  const notifica = useNotifications();
  const isMounted = useIsMounted();


  const [rows, setRows] = React.useState<IncidenciaGrid[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);

  // Cuando cambia el estado de los filtros se actualiza
  const [filtrosActuales, setFiltrosActuales] = React.useState({
    fechaInicio: null as dayjs.Dayjs | null,
    fechaFin: null as dayjs.Dayjs | null,
    estados: [EstadoIncidencia.Solicitud, EstadoIncidencia.ErrorResolver],
    supervisor: false,
    usuarioId: null as number | null
  });

  const actualizarProximoEstado = React.useCallback(
    (id: number,
      nuevoEstado: EstadoIncidencia | undefined,
      motivo: string | null = null) => {
      setRows(prevRows =>
        prevRows.map(row =>
          row.id === id ? {
            ...row,
            accion: nuevoEstado,
            motivoRechazo: motivo
          } : row
        )
      );
    },
    []
  );

  const incidenciaActions: IncidenciaAction[] = React.useMemo(
    () => [
      {
        icon: <CheckIcon />,
        label: 'Aprobar',
        tooltip: 'Marcar para aprobar',
        onClick: (row: IncidenciaGrid) => {
          actualizarProximoEstado(row.id, EstadoIncidencia.Resolver);
        },
        show: (row: IncidenciaGrid) => row.estado === EstadoIncidencia.Solicitud
      },
      {
        icon: <CloseIcon />,
        label: 'Rechazar',
        tooltip: 'Marcar para rechazar motivando',
        onClick: (row: IncidenciaGrid) => {
          dialogo.prompt( // Usa diálogo
            'Motivo del rechazo (máx. 200 caracteres)',
            { title: 'Rechazar' })
            .then(
              motivo => {
                actualizarProximoEstado( // Usa actualizarProximoEstado
                  row.id,
                  EstadoIncidencia.Rechazar,
                  motivo ? motivo.slice(0, 200) : null);
              }
            )
        },
        show: (row: IncidenciaGrid) => row.estado === EstadoIncidencia.Solicitud
      },
      {
        icon: <ReplayIcon />,
        label: 'Reintentar',
        tooltip: 'Marcar para volver a procesar',
        onClick: (row: IncidenciaGrid) => {
          row.motivoRechazo = null;
          actualizarProximoEstado(row.id, EstadoIncidencia.Resolver); // Usa actualizarProximoEstado
        },
        show: (row: IncidenciaGrid) =>
          row.estado === EstadoIncidencia.ErrorResolver
      },
      {
        icon: <CancelIcon />,
        label: 'Cancelar',
        tooltip: 'Cancelar marca',
        onClick: (row: IncidenciaGrid) => {
          row.motivoRechazo = null;
          actualizarProximoEstado(row.id, undefined); // Usa actualizarProximoEstado
        },
        // Muestra si la acción (propiedad de la fila) no es undefined
        // Esta propiedad viene del prop rows del componente, por lo que su valor
        // se actualiza en el scope. Sin embargo, en useMemo/useCallback solo 
        // necesitamos la FUNCIÓN para que esté actualizada.
        show: (row: IncidenciaGrid) => row.accion != undefined
      },
    ],
    [actualizarProximoEstado, dialogo]
  );


  // Genera una entidad con todas las incidencias a procesar,
  // la envía para procesar y refresca el grid con las entidades
  // actualizadas.
  const handleProcesar = React.useCallback(async () => {
    let gridData: IncidenciaGrid[] = []
    setIsLoading(true);

    const incsProceso = rows
      .filter(incidencia => incidencia.accion !== undefined)
      .map(incidencia =>
        new IncidenciaProcesoDTO(
          incidencia.id,
          incidencia.accion!,
          incidencia.motivoRechazo ? incidencia.motivoRechazo.trim() : null
        )
      );

    if (incsProceso.length === 0) {
      notifica.show(
        'No hay acciones definidas para gestionar las incidencias',
        {
          severity: 'warning',
          autoHideDuration: 5000,
        },
      );

      setIsLoading(false);
      return;
    }

    try {
      const data = await api().inc.procesar(
        usuario.id,
        incsProceso,
        filtrosActuales.fechaInicio,
        filtrosActuales.fechaFin,
        filtrosActuales.estados,
        filtrosActuales.supervisor,
        filtrosActuales.usuarioId
      );

      gridData = data.incs.map(inc => ({
        ...inc,
        accion: undefined,
        errorServidor: data.inc_erroneas.includes(inc.id)
      }));

      if (data.inc_erroneas.length > 0) {
        notifica.show(
          `Las incidencias marcadas no han podido ser procesadas 
          por errores fatales. Inténtelo de nuevo, y si vuelve a
          ocurrir consulte con el administrador`,
          {
            severity: 'warning',
            autoHideDuration: 10000,
          },
        );
      } else {
        notifica.show(
          'Incidencias procesadas satisfactóriamente',
          {
            severity: 'success',
            autoHideDuration: 5000,
          },
        );
      }
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('incidencias.procesar', error);
        notifica.show(
          'Error inesperado al cargar la lista de incidencias procesadas',
          {
            severity: 'error',
            autoHideDuration: 5000,
          },
        );
      }
    }

    if (isMounted.current) {
      setRows(gridData);
      setIsLoading(false);
    };
  }, [rows, setIsLoading, filtrosActuales, usuario.id, notifica, setRows]);

  const estadosFiltroMemo = React.useMemo(() => [
    EstadoIncidencia.Solicitud,
    EstadoIncidencia.ErrorResolver,
  ], []);

  return (
    <PageContainer
      title={'Revisión incidencias'}
      actions={
        <Stack direction="row" alignItems="center" spacing={1}>
          <Button
            variant="contained"
            startIcon={<SyncIcon />}
            disabled={isLoading}
            onClick={handleProcesar}
          >
            PROCESAR
          </Button>
        </Stack>
      }
    >
      <Box sx={FULL_HEIGHT_WIDTH}>
        <IncidenciaList
          estadosFiltro={estadosFiltroMemo}
          columnaAccion
          actions={incidenciaActions}
          rows={rows}
          setRows={setRows}
          isLoading={isLoading}
          setIsLoading={setIsLoading}
          onFilterChange={setFiltrosActuales}
        />
      </Box>
    </PageContainer>
  );
}