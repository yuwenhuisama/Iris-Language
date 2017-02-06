#include "IrisComponents/IrisExpressions/IrisCastExpression.h"
#include "IrisInstructorMaker.h"


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
