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
import { api } from '../api/usuarios';
import { UsuarioDTO } from '../modelos/dto';
import { Usuario } from '../modelos/usuarios';

const INITIAL_FORM_VALUES: Partial<UsuarioFormState['values']> = {
};


export default function UsuarioPassword() {
  const { id } = useParams();

  const navegar = useNavigate();
  const notifica = useNotifications();

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
      const validateField = async (
        values: Partial<UsuarioFormState['values']>) => {
        const { issues } = validaUsuarioPass(values);

        setFormErrors({
          ...formErrors,
          [name]: issues?.find((issue) => issue.path?.[0] === name)?.message,
        });
      };

      const newFormValues = { ...formValues, [name]: value };

      setFormValues(newFormValues);
      validateField(newFormValues);
    },
    [formValues, formErrors, setFormErrors, setFormValues],
  )

  const handleFormReset = React.useCallback(() => {
    setFormValues(INITIAL_FORM_VALUES);
  }, [setFormValues]);

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
      await api().usuario.actualizar_password(
        Number(id!), formValues.password!);

      notifica.show('Password cambiada satisfactóriamente.', {
        severity: 'success',
        autoHideDuration: 5000,
      });

      navegar('/usuarios');
    } catch (e) {
      if (e instanceof NetErrorControlado) {
        return;
      }

      notifica.show(
        `Error inesperado al cambiar las password. 
        Razón: ${(e as Error).message}`,
        {
          severity: 'error',
          autoHideDuration: 5000,
        },
      );
    }
  }, [formValues, navegar, notifica, setFormErrors, id]);

  return (
    <PageContainer
      title={`Cambio de password del usuario: ${id}`}
      breadcrumbs={
        [{ title: 'Usuarios', path: '/usuarios' }, { title: 'Passworrd' }]
      }
    >
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
