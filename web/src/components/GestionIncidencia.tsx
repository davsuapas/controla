import Box from '@mui/material/Box';
import CheckIcon from '@mui/icons-material/Check';
import PageContainer from './PageContainer';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import IncidenciaList, { IncidenciaAction, IncidenciaGrid } from './IncidenciaList';
import { EstadoIncidencia, Incidencia } from '../modelos/incidencias';
import React from 'react';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import dayjs from 'dayjs';
import { crearModalInfoSolicitudProps, InfoSolicitud, ModalInfoSolicitud } from './SolicitudIncidencia';
import { NetErrorControlado } from '../net/interceptor';
import { logError } from '../error';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { api } from '../api/fabrica';

// Permite gestionar y consultar las incidencias. 
// La gestión permite volver a solicitar incidencias rechazadas
// o bien que contienen algún conflicto.
// Se vuelve a crear una solicitud volviendo a pedir la
// info de solicitud.
export default function GestionIncidencia(props: { supervisor?: boolean }) {
  const notifica = useNotifications();
  const usuario = useUsuarioLogeado().getUsrLogeado();

  const [rows, setRows] = React.useState<IncidenciaGrid[]>([]);
  const [row, setRow] = React.useState<IncidenciaGrid | undefined>(undefined);
  const [isLoading, setIsLoading] = React.useState(false);

  // Estados para el modal con la información de la solictud
  const [modalOpenInfo, setModalOpenInfo] = React.useState(false);

  // Actualiza la fila
  const actualizarRegistro = (rowu: IncidenciaGrid) => {
    setRows(prevRows => prevRows.map(row => row.id === rowu.id ? rowu : row));
  };

  const incidenciaActions: IncidenciaAction[] = [
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
  ];

  // Procesa la nueva solicitud en base a el estado anterior
  const procesarSolicitud = async (
    info: InfoSolicitud,
    row: IncidenciaGrid) => {
    setIsLoading(true);

    try {
      const inc = await api().inc.cambiarIncidenciaASolictud(
        Incidencia.crearSolicitudFromEstado(
          row.id,
          row.estado,
          info.horaEntrada ?? null,
          info.horaSalida ?? null,
          usuario.id,
          info.motivo,
        )
      );

      actualizarRegistro(IncidenciaGrid.fromIncidencia(inc));
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('gestion-incidencia.solicitar-incidencia', error);
        notifica.show(
          'Error inesperado al volver a realizar una solicitud de incidencia',
          {
            severity: 'error',
            autoHideDuration: 5000,
          }
        );
      }
    }

    setIsLoading(false);
  };

  // Permite abrir un formalario para corregir marcajes
  // mediante solicitud
  const abrirModalInfo = (row: IncidenciaGrid) => {
    setRow(row);
    setModalOpenInfo(true);
  };

  // Cierra la modal mediante un borón aceptar y otro cancelar
  const cerrarModalInfo = React.useCallback(
    (info: InfoSolicitud | undefined) => {
      setModalOpenInfo(false);

      if (info && row) {
        procesarSolicitud(info, row);
      }

      setRow(undefined);
    },
    [row]);

  return (
    <PageContainer title={'Gestor/Consultas incidencias'}>
      <Box sx={FULL_HEIGHT_WIDTH}>
        <IncidenciaList
          estadosFiltro={[
            EstadoIncidencia.Solicitud,
            EstadoIncidencia.Conflicto,
            EstadoIncidencia.Rechazada,
            EstadoIncidencia.Resuelta,
          ]}
          usuarioFiltro={props.supervisor ? 0 : usuario.id}
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