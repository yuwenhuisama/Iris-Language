#ifndef _H_IRISCOMPILERDEV_
#define _H_IRISCOMPILERDEV_

#ifdef _MSC_VER
#pragma warning (push)
#pragma warning (disable : 4005)
#include <intsafe.h>
#include <stdint.h>
#pragma warning (pop)
#endif

#include <sstream>
using namespace std;

#include "IrisComponents/LexYacc/unistd.h"

#include "IrisComponents/IrisComponentsDefines.h"

// Expression
#include "IrisComponents/IrisExpressions/IrisArrayExpression.h"
#include "IrisComponents/IrisExpressions/IrisBinaryExpression.h"
//#include "IrisExpression.h"
#include "IrisComponents/IrisExpressions/IrisFieldExpression.h"
#include "IrisComponents/IrisExpressions/IrisFunctionCallExpression.h"
#include "IrisComponents/IrisExpressions/IrisHashExpression.h"
#include "IrisComponents/IrisExpressions/IrisIdentifierExpression.h"
#include "IrisComponents/IrisExpressions/IrisIndexExpression.h"
#include "IrisComponents/IrisExpressions/IrisInstantValueExpression.h"
#include "IrisComponents/IrisExpressions/IrisMemberExpression.h"
#include "IrisComponents/IrisExpressions/IrisNativeObjectExpression.h"
//#include "IrisSelfFunctionCallExpression.h"
//#include "IrisSelfMemberExpression.h"
#include "IrisComponents/IrisExpressions/IrisSelfExpression.h"
#include "IrisComponents/IrisExpressions/IrisUnaryExpression.h"
#include "IrisComponents/IrisExpressions/IrisRangeExpression.h"

// Parts
#include "IrisComponents/IrisParts/IrisClosureBlockLiteral.h"
#include "IrisComponents/IrisParts/IrisElseIf.h"
#include "IrisComponents/IrisParts/IrisFieldIdentifier.h"
#include "IrisComponents/IrisParts/IrisFunctionHeader.h"
#include "IrisComponents/IrisParts/IrisHashPair.h"
#include "IrisComponents/IrisParts/IrisSwitchBlock.h"
#include "IrisComponents/IrisParts/IrisWhen.h"

// Statements
#include "IrisComponents/IrisStatements/IrisAuthorityStatement.h"
#include "IrisComponents/IrisStatements/IrisBlockStatement.h"
#include "IrisComponents/IrisStatements/IrisBreakStatement.h"
#include "IrisComponents/IrisStatements/IrisCastStatement.h"
#include "IrisComponents/IrisStatements/IrisClassStatement.h"
#include "IrisComponents/IrisStatements/IrisClassStatement.h"
#include "IrisComponents/IrisStatements/IrisConditionIfStatement.h"
#include "IrisComponents/IrisStatements/IrisContinueStatement.h"
#include "IrisComponents/IrisStatements/IrisForStatement.h"
#include "IrisComponents/IrisStatements/IrisFunctionStatement.h"
#include "IrisComponents/IrisStatements/IrisGetterStatement.h"
#include "IrisComponents/IrisStatements/IrisGroanStatement.h"
#include "IrisComponents/IrisStatements/IrisGSetterStatement.h"
#include "IrisComponents/IrisStatements/IrisInterfaceFunctionStatement.h"
#include "IrisComponents/IrisStatements/IrisInterfaceStatement.h"
#include "IrisComponents/IrisStatements/IrisLoopIfStatement.h"
#include "IrisComponents/IrisStatements/IrisModuleStatement.h"
#include "IrisComponents/IrisStatements/IrisNormalStatement.h"
#include "IrisComponents/IrisStatements/IrisOrderStatement.h"
#include "IrisComponents/IrisStatements/IrisReturnStatement.h"
#include "IrisComponents/IrisStatements/IrisSetterStatement.h"
//#include "IrisStatement.h"
#include "IrisComponents/IrisStatements/IrisSuperStatement.h"
#include "IrisComponents/IrisStatements/IrisSwitchStatement.h"

// Unil
#include "IrisUnil/IrisBlock.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisUnil/IrisList.h"

#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"

#endif
