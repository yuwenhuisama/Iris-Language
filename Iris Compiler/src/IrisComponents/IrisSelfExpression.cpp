#include "IrisComponents/IrisExpressions/IrisSelfExpression.h"
#include "IrisInstructorMaker.h"


bool IrisSelfExpression::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pMaker->load_self();
	return true;
}

IrisSelfExpression::IrisSelfExpression()
{
	m_bSelfExpressionFlag = true;
}


IrisSelfExpression::~IrisSelfExpression()
{
}
