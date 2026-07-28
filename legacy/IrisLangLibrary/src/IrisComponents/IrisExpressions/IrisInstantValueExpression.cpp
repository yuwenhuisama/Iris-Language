#include "IrisComponents/IrisExpressions/IrisInstantValueExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"


IrisInstantValueExpression::IrisInstantValueExpression(IrisInstantValueType eType) : m_eType(eType)
{
}


bool IrisInstantValueExpression::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	switch (m_eType)
	{
	case IrisInstantValueType::Nil:
		pMaker->load_nil();
		break;
	case IrisInstantValueType::True:
		pMaker->load_true();
		break;
	case IrisInstantValueType::False:
		pMaker->load_false();
		break;
	default:
		break;
	}

	return true;
}

IrisInstantValueExpression::~IrisInstantValueExpression()
{
}
