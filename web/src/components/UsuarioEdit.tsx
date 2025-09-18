import * as React from 'react';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import { useNavigate, useParams } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import UsuarioForm, {
  Presentacion,
  setPropGeneralesUsuario,
  validaUsuario,
  type FormFieldValue,
  type UsuarioFormState,
} from './UsuarioForm';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { Usuario } from '../modelos/usuarios';
import { UsuarioDTO } from '../modelos/dto';
import { api } from '../api/usuarios';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';


function UsuarioEditForm({
  initialValues,
  onSubmit,
}: {
  initialValues: Partial<UsuarioFormState['values']>;
  onSubmit: (formValues: Partial<UsuarioFormState['values']>) => Promise<void>;
}) {
  const navegar = useNavigate();
  const notifica = useNotifications();

  const [formState, setFormState] = React.useState<UsuarioFormState>(() => ({
    values: initialValues,
    errors: {},
  }));
  const formValues = formState.values;
  const formErrors = formState.errors;

  // Maneja los cambios en los campos
  const setFormValues = React.useCallback(
    (newFormValues: Partial<UsuarioFormState['values']>) => {
      setFormState((previousState) => ({
        ...previousState,
        values: newFormValues,
      }));
    },
    [],
  );

  // Maneja los errores de los campos
  const setFormErrors = React.useCallback(
    (newFormErrors: Partial<UsuarioFormState['errors']>) => {
      setFormState((previousState) => ({
        ...previousState,
        errors: newFormErrors,
      }));
    },
    [],
  );

  // Evento que lanza el cambio de un campo y la validación
  const handleFormFieldChange = React.useCallback(
    (name: keyof UsuarioFormState['values'], value: FormFieldValue) => {
      const newFormValues = {
        ...formValues, [name]: setPropGeneralesUsuario(name, value)
      };

      setFormValues(newFormValues);

      const validateField = async (values: Partial<UsuarioFormState['values']>) => {
        const { issues } = validaUsuario(values);
        setFormErrors({
          ...formErrors,
          [name]: issues?.find((issue) => issue.path?.[0] === name)?.message,
        });
      };

      validateField(newFormValues);
    },
    [formValues, formErrors, setFormErrors, setFormValues],
  );

  const handleFormReset = React.useCallback(() => {
    setFormValues(initialValues);
  }, [initialValues, setFormValues]);

  // Maneja el envío del formulario
  const handleFormSubmit = React.useCallback(async () => {
    const { issues } = validaUsuario(formValues);

    if (issues && issues.length > 0) {
      setFormErrors(
        Object.fromEntries(
          issues.map((issue) => [issue.path?.[0], issue.message])),
      );
      return;
    }

    setFormErrors({});

    try {
      await onSubmit(formValues);

      notifica.show('Usuario actualizado satisfactóriamente.', {
        severity: 'success',
        autoHideDuration: 5000,
      });

      navegar('/usuarios');
    } catch (editError) {
      if (editError instanceof NetErrorControlado) {
        return;
      }

      notifica.show(
        `Error inesperado al actualizado el usuario. 
        Razón: ${(editError as Error).message}`,
        {
          severity: 'error',
          autoHideDuration: 5000,
        },
      );
    }
  }, [formValues, navegar, notifica, onSubmit, setFormErrors]);

  return (
    <UsuarioForm
      formState={formState}
      onFieldChange={handleFormFieldChange}
      onSubmit={handleFormSubmit}
      onReset={handleFormReset}
      submitButtonLabel="ACTUALIZAR"
      presentacion={Presentacion.SIN_PASSWORD}
    />
  );
}

export default function UsuarioEdit() {
  const { id } = useParams();
  const { getUsrLogeado } = useUsuarioLogeado()

  const [usuario, setUsuario] = React.useState<Usuario | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);

  const loadData = React.useCallback(async () => {
    setError(null);
    setIsLoading(true);

    try {
      const showData = await api().usuarios.usuario(id ?? '');

      setUsuario(showData);
    } catch (showDataError) {
      setError(showDataError as Error);
    }
    setIsLoading(false);
  }, [id]);

  React.useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSubmit = React.useCallback(
    async (formValues: UsuarioFormState['values']) => {

      const usrLog = getUsrLogeado()
      let usr = formValues as Usuario
      usr.autor = usrLog.id

      return api().usuarios.actualizar_usuario(
        UsuarioDTO.fromUsuario(usr),
      );
    },
    [],
  );

  const renderEdit = React.useMemo(() => {
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
    if (error) {
      return (
        <Box sx={{ flexGrow: 1 }}>
          <Alert severity="error">{error.message}</Alert>
        </Box>
      );
    }

    return usuario ? (
      <UsuarioEditForm initialValues={usuario} onSubmit={handleSubmit} />
    ) : null;
  }, [isLoading, error, usuario, handleSubmit]);

  return (
    <PageContainer
      title={`Edición del usuario: ${id}`}
      breadcrumbs={
        [{ title: 'Usuarios', path: '/usuarios' }, { title: 'Edición' }]
      }
    >
      <Box sx={{ display: 'flex', flex: 1 }}>{renderEdit}</Box>
    </PageContainer>
  );
}
