import Box from '@mui/material/Box';
import CheckIcon from '@mui/icons-material/Check';
import CancelIcon from '@mui/icons-material/Cancel';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import IncidenciaList, { IncidenciaAction, IncidenciaGrid } from './IncidenciaList';
import { EstadoIncidencia, Incidencia } from '../modelos/incidencias';
import React from 'react';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import dayjs from 'dayjs';
import { crearModalInfoSolicitudProps, InfoSolicitud, ModalInfoSolicitud } from './IncidenciaSolicitud';
import { NetErrorControlado } from '../net/interceptor';
import { logError } from '../error';
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { api } from '../api/fabrica';
import { useIsMounted } from '../hooks/useComponentMounted';

// Permite gestionar y consultar las incidencias. 
// La gestión permite volver a solicitar incidencias rechazadas
// o bien que contienen algún conflicto.
// Se vuelve a crear una solicitud volviendo a pedir la
// info de solicitud.
type EjecutarSolicitud = (
) => Promise<Incidencia>;

export default function GestionIncidencia() {
  const notifica = useNotifications();
  const usuario = useUsuarioLogeado().getUsrLogeado();
  const dialogo = useDialogs();
  const isMounted = useIsMounted();

  const [rows, setRows] = React.useState<IncidenciaGrid[]>([]);
  const [row, setRow] = React.useState<IncidenciaGrid | undefined>(undefined);
  const [isLoading, setIsLoading] = React.useState(false);
  const [modalOpenInfo, setModalOpenInfo] = React.useState(false);

  const actualizarRegistro = React.useCallback((rowu: IncidenciaGrid) => {
    setRows(prevRows => prevRows.map(row => row.id === rowu.id ? rowu : row));
  }, []);

  const cambiarEstado = React.useCallback(
    async (ejecutarCambio: EjecutarSolicitud) => {
      setIsLoading(true);

      try {
        const inc = await ejecutarCambio();

        if (isMounted.current) {
          actualizarRegistro(IncidenciaGrid.fromIncidencia(inc));
        };
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('gestion-incidencia.solicitar-incidencia', dialogo?.alert, error);
          notifica.show(
            'Error inesperado al volver a realizar una solicitud de incidencia',
            { severity: 'error', autoHideDuration: 5000 }
          );
        }
      }
      if (isMounted.current) {
        setIsLoading(false);
      }
    },
    [actualizarRegistro, dialogo?.alert, isMounted, notifica]
  );

  const abrirModalInfo = React.useCallback((row: IncidenciaGrid) => {
    setRow(row);
    setModalOpenInfo(true);
  }, []);

  const incidenciaActions: IncidenciaAction[] = React.useMemo(
    () => [
      {
        icon: <CheckIcon />,
        label: 'Volver a solicitar',
        tooltip: 'Volver a solicitar',
        onClick: (row: IncidenciaGrid) => {
          abrirModalInfo(row);
        },
        show: (row: IncidenciaGrid) =>
          row.estado === EstadoIncidencia.Conflicto ||
          row.estado === EstadoIncidencia.Rechazada
      },
      {
        icon: <CancelIcon />,
        label: 'Cancelar',
        tooltip: 'Cancelar solicitud',
        onClick: (row: IncidenciaGrid) => {
          cambiarEstado(async () =>
            api().inc.cambiarIncidenciaACancelada(
              row.id,
              dayjs()
            ));
        },
        show: (row: IncidenciaGrid) =>
          row.estado !== EstadoIncidencia.Resuelta &&
          row.estado !== EstadoIncidencia.Cancelada
      },
    ],
    [abrirModalInfo]
  );

  const cerrarModalInfo = React.useCallback(
    (info: InfoSolicitud | undefined) => {
      setModalOpenInfo(false);

      if (info && row) {
        cambiarEstado(async () =>
          api().inc.cambiarIncidenciaASolictud(
            Incidencia.crearSolicitudFromEstado(
              row.id,
              row.estado,
              info.horaEntrada ?? null,
              info.horaSalida ?? null,
              usuario.id,
              info.motivo,
            )
          )
        );
      }

      setRow(undefined);
    },
    [cambiarEstado, row, usuario.id]
  );

  // Evita que se vuelve a crear en los renderizados
  // Además, como estadosFiltro es usado en dependencias
  // puede provocar que aunque no cambie el contenido
  // si cambia el puntero y react vuelve a construir
  // la función y provoca recursividad.
  const estadosFiltroMemo = React.useMemo(() => [
    EstadoIncidencia.Solicitud,
    EstadoIncidencia.Conflicto,
    EstadoIncidencia.Rechazada,
    EstadoIncidencia.Resuelta,
    EstadoIncidencia.Cancelada,
  ], []);

  return (
    <PageContainer title={'Gestor incidencias'}>
      <Box sx={FULL_HEIGHT_WIDTH}>
        <IncidenciaList
          estadosFiltro={estadosFiltroMemo}
          usuarioFiltro={usuario.id}
          actions={incidenciaActions}
          rows={rows}
          setRows={setRows}
          isLoading={isLoading}
          setIsLoading={setIsLoading}
        />

        {row && (
          <ModalInfoSolicitud
            open={modalOpenInfo}
            onClose={cerrarModalInfo}
            fecha={dayjs(row.fecha)}
            {...crearModalInfoSolicitudProps(
              dayjs(row.fecha), row.tipo,
              row?.horaInicio ? row.horaInicio : undefined,
              row?.horaFin ? row.horaFin : undefined)!}
          />
        )}
      </Box>
    </PageContainer>
  );
}