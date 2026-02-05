import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import { DataGrid, GridActionsCellItem, GridColDef } from '@mui/x-data-grid';
import AddIcon from '@mui/icons-material/Add';
import RefreshIcon from '@mui/icons-material/Refresh';
import EditIcon from '@mui/icons-material/Edit';
import DeleteIcon from '@mui/icons-material/Delete';
import DateRangeIcon from '@mui/icons-material/DateRange';
import { useNavigate } from 'react-router';
import PageContainer from './PageContainer';
import { api } from '../api/fabrica';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';
import { Calendario } from '../modelos/calendario';
import { useDialogs } from '../hooks/useDialogs/useDialogs';

export default function CalendarioList() {
  const navegar = useNavigate();
  const notifica = useNotifications();
  const isMounted = useIsMounted();
  const { confirm } = useDialogs();

  const [rows, setRows] = React.useState<Calendario[]>([]);
  const [isLoading, setIsLoading] = React.useState(true);

  const loadData = React.useCallback(async () => {
    setIsLoading(true);
    try {
      const listData = await api().calendar.calendarios();
      if (isMounted.current) setRows(listData);
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('calendario-listar.cargar', error);
        notifica.show('Error inesperado al cargar los calendarios', {
          severity: 'error',
          autoHideDuration: 5000,
        });
      }
    } finally {
      if (isMounted.current) setIsLoading(false);
    }
  }, [isMounted, notifica]);

  React.useEffect(() => {
    loadData();
  }, [loadData]);

  const handleRefresh = React.useCallback(() => {
    if (!isLoading) loadData();
  }, [isLoading, loadData]);

  const handleEditClick = React.useCallback((id: number) => () => {
    navegar(`/calendarios/${id}`);
  }, [navegar]);

  const handleDeleteClick = React.useCallback((id: number) => async () => {
    const confirmed = await confirm('¿Está seguro de que desea eliminar este calendario?', {
      title: 'Eliminar calendario',
      okText: 'ELIMINAR',
      severity: 'warning',
    });

    if (confirmed) {
      try {
        await api().calendar.eliminarCalendario(id);

        notifica.show('Calendario eliminado satisfactoriamente', {
          severity: 'success',
          autoHideDuration: 3000,
        });

        await loadData();
      } catch (error) {
        if (!(error instanceof NetErrorControlado)) {
          logError('calendario-listar.eliminar', error);
          notifica.show('Error inesperado al eliminar el calendario', {
            severity: 'error',
            autoHideDuration: 5000,
          });
        }
      }
    }
  }, [notifica, loadData, confirm]);

  const handleDatesClick = React.useCallback((id: number) => () => {
    navegar(`/calendarios/${id}/fechas`);
  }, [navegar]);

  const handleCreateClick = React.useCallback(() => {
    navegar('/calendarios/nuevo');
  }, [navegar]);

  const columns = React.useMemo<GridColDef<Calendario>[]>(
    () => [
      { field: 'id', headerName: 'ID', width: 90 },
      { field: 'nombre', headerName: 'Nombre', flex: 1, minWidth: 200 },
      { field: 'descripcion', headerName: 'Descripción', flex: 2, minWidth: 300 },
      {
        field: 'actions',
        type: 'actions',
        width: 120,
        getActions: ({ id }) => [
          <Tooltip title="Editar calendario" key={`edit-${id}`}>
            <GridActionsCellItem
              icon={<EditIcon />}
              label="Editar"
              onClick={handleEditClick(id as number)}
            />
          </Tooltip>,
          <Tooltip title="Eliminar calendario" key={`delete-${id}`}>
            <GridActionsCellItem
              icon={<DeleteIcon />}
              label="Eliminar"
              onClick={handleDeleteClick(id as number)}
            />
          </Tooltip>,
          <Tooltip title="Gestionar fechas" key={`dates-${id}`}>
            <GridActionsCellItem
              icon={<DateRangeIcon />}
              label="Fechas"
              onClick={handleDatesClick(id as number)}
            />
          </Tooltip>,
        ],
      },
    ],
    [handleEditClick, handleDeleteClick, handleDatesClick],
  );

  const pageTitle = 'Calendarios';

  return (
    <PageContainer
      title={pageTitle}
      breadcrumbs={[{ title: pageTitle }]}
      actions={
        <Stack direction="row" alignItems="center" spacing={1}>
          <Tooltip title="Recargar datos" placement="right" enterDelay={1000}>
            <IconButton size="small" aria-label="refresh" onClick={handleRefresh}>
              <RefreshIcon />
            </IconButton>
          </Tooltip>
          <Button variant="contained" onClick={handleCreateClick} startIcon={<AddIcon />}>
            NUEVO
          </Button>
        </Stack>
      }
    >
      <Box sx={FULL_HEIGHT_WIDTH}>
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