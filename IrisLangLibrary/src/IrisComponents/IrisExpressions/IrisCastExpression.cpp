#include "IrisComponents/IrisExpressions/IrisCastExpression.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"

bool IrisCastExpression::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pMaker->load_cast();
	return true;
}

IrisCastExpression::IrisCastExpression()
{
}


IrisCastExpression::~IrisCastExpression()
{
}

bool IrisCastExpression::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	if (!pCompiler->UpperWithBlock()) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::CastExpressionIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "cast Expression can only be used in a Closure Block or Method Block.");
		return false;
	}
	
	return true;
}
