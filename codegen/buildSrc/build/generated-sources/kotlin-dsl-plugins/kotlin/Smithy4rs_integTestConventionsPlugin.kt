/**
 * Precompiled [smithy4rs.integ-test-conventions.gradle.kts][Smithy4rs_integ_test_conventions_gradle] script plugin.
 *
 * @see Smithy4rs_integ_test_conventions_gradle
 */
public
class Smithy4rs_integTestConventionsPlugin : org.gradle.api.Plugin<org.gradle.api.Project> {
    override fun apply(target: org.gradle.api.Project) {
        try {
            Class
                .forName("Smithy4rs_integ_test_conventions_gradle")
                .getDeclaredConstructor(org.gradle.api.Project::class.java, org.gradle.api.Project::class.java)
                .newInstance(target, target)
        } catch (e: java.lang.reflect.InvocationTargetException) {
            throw e.targetException
        }
    }
}
