import * as React from 'react';
import { useNavigate } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import UsuarioForm, {
  concatenarValidaciones,
  Presentacion,
  setPropGeneralesUsuario,
  validaUsuario,
  validaUsuarioPass,
  type FormFieldValue,
  type UsuarioFormState,
} from './UsuarioForm';
import PageContainer from './PageContainer';
import dayjs from 'dayjs';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import { UsuarioOutDTO } from '../modelos/dto';
import { Usuario } from '../modelos/usuarios';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { logError } from '../error';

const INITIAL_FORM_VALUES: Partial<UsuarioFormState['values']> = {
  activo: dayjs(),
};

export default function UsuarioCreate() {
  const navegar = useNavigate();
  const notifica = useNotifications();
  const { getUsrLogeado } = useUsuarioLogeado()

  const [formState, setFormState] = React.useState<UsuarioFormState>(() => ({
    values: INITIAL_FORM_VALUES,
    errors: {},
  }));
  const formValues = formState.values;

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
      setFormState((currentState) => {
        const newFormValues = {
          ...currentState.values,
          [name]: setPropGeneralesUsuario(name, value)
        };

        const validateAndUpdateErrors = () => {
          const { issues } = concatenarValidaciones(
            validaUsuario(newFormValues),
            validaUsuarioPass(newFormValues)
          );

          setFormState((prevState) => ({
            ...prevState,
            errors: {
              ...prevState.errors,
              [name]: issues?.find((issue) => issue.path?.[0] === name)?.message,
            }
          }));
        };

        validateAndUpdateErrors();

        return {
          ...currentState,
          values: newFormValues,
        };
      });
    },
    [],
  );

  const handleFormReset = React.useCallback(() => {
    setFormValues(INITIAL_FORM_VALUES);
  }, []);

  // Maneja el envío del formulario
  const handleFormSubmit = React.useCallback(async () => {
    const { issues } = concatenarValidaciones(
      validaUsuario(formValues), validaUsuarioPass(formValues));

    if (issues && issues.length > 0) {
      setFormErrors(
        Object.fromEntries(
          issues.map((issue) => [issue.path?.[0], issue.message])),
      );

      notifica.show(
        'Imposible crear el usuario. Corriga los errores',
        {
          severity: 'warning',
          autoHideDuration: 5000,
        },
      );

      return;
    }

    setFormErrors({});

    try {
      const usrLog = getUsrLogeado()
      let usr = formValues as Usuario
      usr.autor = usrLog.id

      await api().usuarios.crearUsuario(
        UsuarioOutDTO.fromUsuario(usr),
      );

      notifica.show('Usuario creado satisfactóriamente', {
        severity: 'success',
        autoHideDuration: 5000,
      });

      navegar('/usuarios');
    } catch (error) {
      if (error instanceof NetErrorControlado) {
        return;
      }

      logError('usuario-crear.crear', error);

      notifica.show(
        'Error inesperado al crear el usuario',
        {
          severity: 'error',
          autoHideDuration: 5000,
        },
      );
    }
  }, [formValues, notifica, navegar, getUsrLogeado]);

  return (
    <PageContainer
      title="Nuevo usuario"
      breadcrumbs={
        [{ title: 'Usuarios', path: '/usuarios' }, { title: 'Nuevo' }]
      }
    >
      <UsuarioForm
        formState={formState}
        onFieldChange={handleFormFieldChange}
        onSubmit={handleFormSubmit}
        onReset={handleFormReset}
        submitButtonLabel="CREAR"
        presentacion={Presentacion.FULL}
      />
    </PageContainer>
  );
}
