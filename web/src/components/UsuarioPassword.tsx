import * as React from 'react';
import { useNavigate, useParams } from 'react-router';
import useNotifications from '../hooks/useNotifications/useNotifications';
import UsuarioForm, {
  Presentacion,
  validaUsuarioPass,
  type FormFieldValue,
  type UsuarioFormState,
} from './UsuarioForm';
import PageContainer from './PageContainer';
import { NetErrorControlado } from '../net/interceptor';
import { api } from '../api/fabrica';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { logError } from '../error';

const INITIAL_FORM_VALUES: Partial<UsuarioFormState['values']> = {
};


export default function UsuarioPassword() {
  const { id } = useParams();
  const { getUsrLogeado } = useUsuarioLogeado()

  const usuarioId = Number(id) || getUsrLogeado().id;

  const navegar = useNavigate();
  const notifica = useNotifications();

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
          [name]: value
        };

        const validateField = () => {
          const { issues } = validaUsuarioPass(newFormValues);
          const fieldError = issues?.find((issue) => issue.path?.[0] === name)?.message;

          setFormState((prevState) => ({
            ...prevState,
            errors: {
              ...prevState.errors,
              [name]: fieldError,
            }
          }));
        };

        validateField();

        // Actualizar valores inmediatamente
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
    const { issues } = validaUsuarioPass(formValues);

    if (issues && issues.length > 0) {
      setFormErrors(
        Object.fromEntries(
          issues.map((issue) => [issue.path?.[0], issue.message])),
      );
      return;
    }

    setFormErrors({});

    try {
      await api().usuarios.actualizar_password(
        usuarioId, formValues.password!);

      notifica.show('Password cambiada satisfactóriamente.', {
        severity: 'success',
        autoHideDuration: 5000,
      });

      navegar('/usuarios');
    } catch (error) {
      if (error instanceof NetErrorControlado) {
        return;
      }

      logError('usuariopassword.actualizar', error);

      notifica.show(
        'Error inesperado al modificar las password de el usuario',
        {
          severity: 'error',
          autoHideDuration: 5000,
        },
      );
    }
  }, [formValues, usuarioId]);

  return (
    <PageContainer title={`Cambio de password del usuario: ${usuarioId}`}>
      <UsuarioForm
        formState={formState}
        onFieldChange={handleFormFieldChange}
        onSubmit={handleFormSubmit}
        onReset={handleFormReset}
        submitButtonLabel="CAMBIAR PASSWORD"
        presentacion={Presentacion.SOLO_PASSWORD}
      />
    </PageContainer >
  );
}
