import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import { DataGrid, GridActionsCellItem, GridColDef } from '@mui/x-data-grid';
import AddIcon from '@mui/icons-material/Add';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import { useNavigate, useParams } from 'react-router';
import PageContainer from './PageContainer';
import { api } from '../api/fabrica';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';
import { CalendarioFecha, NombresTipoCalendarioFecha } from '../modelos/calendario';
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import { dateToStr } from '../modelos/formatos';
import { SelectorFechas, SelectorFechasRef } from './SelectorFechas';

export default function CalendarioFechas() {
  const { id: calendarioId } = useParams<{ id: string }>();
  const navegar = useNavigate();
  const notifica = useNotifications();
  const isMounted = useIsMounted();
  const { confirm } = useDialogs();
  const selectorFechasRef = React.useRef<SelectorFechasRef>(null);

  const [rows, setRows] = React.useState<CalendarioFecha[]>([]);
  const [isLoading, setIsLoading] = React.useState(true);

  const loadData = React.useCallback(async () => {
    setIsLoading(true);
    try {
      let fechaInicio = null;
      let fechaFin = null;

      if (selectorFechasRef.current) {
        const data = selectorFechasRef.current.getFormData();
        fechaInicio = data.fechaInicio;
        fechaFin = data.fechaFin;
      }
      const listData = await api().calendar.fechas(
        Number(calendarioId), fechaInicio, fechaFin);
      if (isMounted.current) setRows(listData);
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('calendario-fechas.cargar', error);
        notifica.show('Error inesperado al cargar las fechas del calendario', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    } finally {
      if (isMounted.current) setIsLoading(false);
    }
  }, [calendarioId, isMounted, notifica]);

  React.useEffect(() => {
    loadData();
  }, [loadData]);

  const handleRefresh = React.useCallback(() => {
    if (!isLoading) loadData();
  }, [isLoading, loadData]);

  const handleEditClick = React.useCallback((id: number) => () => {
    navegar(`/calendarios/${calendarioId}/fechas/${id}`);
  }, [navegar, calendarioId]);

  const handleDeleteClick = React.useCallback((id: number) => async () => {
    const confirmed = await confirm('¿Está seguro de que desea eliminar esta fecha?', {
      title: 'Eliminar fecha',
      okText: 'ELIMINAR',
      severity: 'warning',
    });

    if (confirmed) {
      try {
        await api().calendar.eliminarFecha(id);
        notifica.show('Fecha eliminada satisfactoriamente', {
          severity: 'success',
          autoHideDuration: 3000,
        });
        await loadData();
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('calendario-fechas.eliminar', error);
          notifica.show('Error inesperado al eliminar la fecha', {
            severity: 'error',
            autoHideDuration: 5000,
          });
        }
      }
    }
  }, [notifica, loadData, confirm]);

  const handleCreateClick = React.useCallback(() => {
    navegar(`/calendarios/${calendarioId}/fechas/nuevo`);
  }, [navegar, calendarioId]);

  const columns = React.useMemo<GridColDef<CalendarioFecha>[]>(
    () => [
      { field: 'id', headerName: 'ID', width: 90 },
      {
        field: 'fechaInicio',
        headerName: 'Fecha inicio',
        flex: 1,
        minWidth: 150,
        valueGetter: (value) => dateToStr(value)
      },
      {
        field: 'fechaFin',
        headerName: 'Fecha fin',
        flex: 1,
        minWidth: 150,
        valueGetter: (value) => dateToStr(value)
      },
      {
        field: 'tipo',
        headerName: 'Tipo',
        flex: 1,
        minWidth: 200,
        valueGetter: (value) => NombresTipoCalendarioFecha[value]
      },
      {
        field: 'actions',
        type: 'actions',
        width: 120,
        getActions: ({ id }) => [
          <Tooltip title="Editar fecha" key={`edit-${id}`}>
            <GridActionsCellItem
              icon={<EditIcon />}
              label="Editar"
              onClick={handleEditClick(id as number)}
            />
          </Tooltip>,
          <Tooltip title="Eliminar fecha" key={`delete-${id}`}>
            <GridActionsCellItem
              icon={<DeleteIcon />}
              label="Eliminar"
              onClick={handleDeleteClick(id as number)}
            />
          </Tooltip>,
        ],
      },
    ],
    [handleEditClick, handleDeleteClick],
  );

  const pageTitle = 'Fechas del Calendario';

  return (
    <PageContainer
      title={pageTitle}
      breadcrumbs={[{ title: 'Calendarios', path: '/calendarios' }, { title: 'Fechas' }]}
      actions={
        <Stack direction="row" alignItems="center" spacing={1}>
          <Button variant="contained" onClick={handleCreateClick} startIcon={<AddIcon />}>
            NUEVO
          </Button>
        </Stack>
      }
    >
      <Box sx={FULL_HEIGHT_WIDTH}>
        <Grid container spacing={2} sx={{ mb: 2, width: '100%' }}>
          <SelectorFechas
            ref={selectorFechasRef}
            labelUltimosRegistros="Primeras fechas"
          />
          <Grid size={{ xs: 12, sm: 12, md: 2 }}>
            <Button
              variant="contained"
              onClick={handleRefresh}
              disabled={isLoading}
              sx={{ mt: 0.5, width: '100%' }}
            >
              FILTRAR
            </Button>
          </Grid>
        </Grid>
        <DataGrid
          rows={rows}
          columns={columns}
          ignoreDiacritics
          loading={isLoading}
          disableRowSelectionOnClick
          initialState={{
            pagination: { paginationModel: { pageSize: 25, page: 0 } },
          }}
          pageSizeOptions={[25, 50, 100]}
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
    </PageContainer>
  );
}