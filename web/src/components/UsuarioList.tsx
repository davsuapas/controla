import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import {
  DataGrid,
  GridActionsCellItem,
  GridColDef,
  GridEventListener,
  gridClasses,
} from '@mui/x-data-grid';
import AddIcon from '@mui/icons-material/Add';
import RefreshIcon from '@mui/icons-material/Refresh';
import { useNavigate } from 'react-router';
import PageContainer from './PageContainer';
import { Rol, Usuario } from '../modelos/usuarios';
import { api } from '../api/fabrica';
import Chip from '@mui/material/Chip';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import PasswordIcon from '@mui/icons-material/Password';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { logError } from '../error';

export default function UsuarioList() {
  const navegar = useNavigate();
  const notifica = useNotifications();

  const [rowsState, setRowsState] = React.useState<{
    rows: Usuario[];
  }>({
    rows: [],
  });

  const [isLoading, setIsLoading] = React.useState(true);

  // Carga los usuarios
  const loadData = React.useCallback(async () => {
    setIsLoading(true);
    let listData: Usuario[] = [];

    try {
      listData = await api().usuarios.usuarios();
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('usuariolistar.cargar', error);

        notifica.show(
          'Error inesperado al cargar la lista de usuarios',
          {
            severity: 'error',
            autoHideDuration: 5000,
          },
        );
      }
    }

    setRowsState({
      rows: listData,
    });

    setIsLoading(false);
  }, []);

  React.useEffect(() => {
    loadData();
  }, []);

  // Refresca la lista
  const handleRefresh = React.useCallback(() => {
    if (!isLoading) {
      loadData();
    }
  }, [isLoading]);

  // Edita un usuario
  const handleRowClick = React.useCallback<GridEventListener<'rowClick'>>(
    ({ row }) => {
      navegar(`/usuarios/${row.id}`);
    },
    [],
  );

  // Navega para el cambio de password
  const handlePasswordClick = React.useCallback(
    (usuario: Usuario) => () => {
      navegar(`/usuarios/${usuario.id}/password`);
    },
    [],
  );

  // Crea un nuevo usuario
  const handleCreateClick = React.useCallback(() => {
    navegar('/usuarios/nuevo');
  }, []);

  const columns = React.useMemo<GridColDef[]>(
    () => [
      {
        field: 'id',
        headerName: 'ID',
        width: 20
      },
      {
        field: 'dni',
        headerName: 'DNI',
        width: 100
      },
      {
        field: 'nombre',
        headerName: 'NOMBRE',
        width: 300,
        valueGetter: (_, row) => row.nombreCompleto()
      },
      {
        field: 'roles',
        headerName: 'ROLES',
        width: 320,
        sortable: false,
        renderCell: (params) => {
          const roles: [Rol] = params.row.roles;

          const muchosRoles = roles.length > 4;

          return (
            <div style={{
              display: muchosRoles ? 'flex' : undefined,
              flexWrap: 'wrap',
            }}>
              {roles.map((rol) => (
                <Chip
                  key={rol.id}
                  label={rol.nombre}
                  size="small"
                  variant="outlined"
                />
              ))}
            </div>
          );
        }
      },
      {
        field: 'activo',
        headerName: 'ACTIVO',
        width: 120,
        filterable: false,
        sortable: false,
        renderCell: (params) => {
          const valor = params.row.activoToStr();
          const esNoActivo = valor === "No activo";

          return (
            <Chip
              label={valor}
              size="small"
              color={esNoActivo ? "error" : "success"}
              variant={esNoActivo ? "outlined" : "filled"}
              sx={{
                borderRadius: '16px',
                fontSize: '0.75rem',
                fontWeight: esNoActivo ? 500 : 400,
                minWidth: '80px'
              }}
            />
          );
        }
      },
      {
        field: 'inicio',
        headerName: 'PRIMER ACESSO',
        width: 130,
        filterable: false,
        sortable: false,
        renderCell: (params) => {
          const valor = params.row.inicioToStr();
          const esNoLogeado = valor === "No logeado";

          return (
            <Chip
              label={valor}
              size="small"
              color={esNoLogeado ? "error" : "success"}
              variant={esNoLogeado ? "outlined" : "filled"}
              sx={{
                borderRadius: '16px',
                fontSize: '0.75rem',
                fontWeight: esNoLogeado ? 500 : 400,
                minWidth: '80px'
              }}
            />
          );
        }
      },
      {
        field: 'actions',
        type: 'actions',
        flex: 1,
        align: 'right',
        getActions: ({ row }) => [
          <Tooltip title="Cambiar password" key="password-tooltip">
            <GridActionsCellItem
              key="password-item"
              icon={<PasswordIcon />}
              label="Cambiar passowrd"
              onClick={handlePasswordClick(row)}
            />
          </Tooltip>,
        ],
      },
    ],
    [handlePasswordClick],
  );

  const pageTitle = 'Usuarios';

  return (
    <PageContainer
      title={pageTitle}
      breadcrumbs={[{ title: pageTitle }]}
      actions={
        <Stack direction="row" alignItems="center" spacing={1}>
          <Tooltip title="Reload data" placement="right" enterDelay={1000}>
            <div>
              <IconButton size="small" aria-label="refresh" onClick={handleRefresh}>
                <RefreshIcon />
              </IconButton>
            </div>
          </Tooltip>
          <Button
            variant="contained"
            onClick={handleCreateClick}
            startIcon={<AddIcon />}
          >
            CREAR
          </Button>
        </Stack>
      }
    >
      <Box sx={FULL_HEIGHT_WIDTH}>
        <DataGrid
          rows={rowsState.rows}
          columns={columns}
          ignoreDiacritics
          disableRowSelectionOnClick
          pageSizeOptions={[]}
          initialState={{
            pagination: {
              paginationModel: { pageSize: 25, page: 0 },
            },
          }}
          onRowClick={handleRowClick}
          loading={isLoading}
          showToolbar
          sx={{
            height: '100%',
            [`& .${gridClasses.columnHeader}, & .${gridClasses.cell}`]: {
              outline: 'transparent',
            },
            [`& .${gridClasses.columnHeader}:focus-within, & .${gridClasses.cell}:focus-within`]:
            {
              outline: 'none',
            },
            [`& .${gridClasses.row}:hover`]: {
              cursor: 'pointer',
            },
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
      </Box>
    </PageContainer>
  );
}
