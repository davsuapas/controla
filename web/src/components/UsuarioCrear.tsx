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
import { api } from '../api/usuarios';
import { UsuarioDTO } from '../modelos/dto';
import { Usuario } from '../modelos/usuarios';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';

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

      const validateField = async (
        values: Partial<UsuarioFormState['values']>) => {
        const { issues } = concatenarValidaciones(
          validaUsuario(values), validaUsuarioPass(values));

        setFormErrors({
          ...formErrors,
          [name]: issues?.find((issue) => issue.path?.[0] === name)?.message,
        });
      };

      validateField(newFormValues);
    },
    [formValues, formErrors, setFormErrors, setFormValues],
  )

  const handleFormReset = React.useCallback(() => {
    setFormValues(INITIAL_FORM_VALUES);
  }, [setFormValues]);

  // Maneja el envío del formulario
  const handleFormSubmit = React.useCallback(async () => {
    const { issues } = concatenarValidaciones(
      validaUsuario(formValues), validaUsuarioPass(formValues));

    if (issues && issues.length > 0) {
      setFormErrors(
        Object.fromEntries(
          issues.map((issue) => [issue.path?.[0], issue.message])),
      );
      return;
    }

    setFormErrors({});

    try {
      const usrLog = getUsrLogeado()
      let usr = formValues as Usuario
      usr.autor = usrLog.id

      await api().usuarios.crear_usuario(
        UsuarioDTO.fromUsuario(usr),
      );

      notifica.show('Usuario creado satisfactóriamente.', {
        severity: 'success',
        autoHideDuration: 5000,
      });

      navegar('/usuarios');
    } catch (e) {
      if (e instanceof NetErrorControlado) {
        return;
      }

      notifica.show(
        `Error inesperado al crear el usuario. 
        Razón: ${(e as Error).message}`,
        {
          severity: 'error',
          autoHideDuration: 5000,
        },
      );
    }
  }, [formValues, navegar, notifica, setFormErrors]);

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
