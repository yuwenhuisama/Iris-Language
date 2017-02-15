#include "IrisComponents/IrisExpressions/IrisSelfExpression.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"

bool IrisSelfExpression::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pMaker->load_self();
	return true;
}

IrisSelfExpression::IrisSelfExpression()
{
}


IrisSelfExpression::~IrisSelfExpression()
{
}

bool IrisSelfExpression::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();

	if (!pCompiler->UpperWithBlock()) {

		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::SelfExpressionIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "self Expression can only be used in Closure Block or Method Block.");

		return false;
	}

	return true;
}
