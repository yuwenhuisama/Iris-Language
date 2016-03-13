#ifndef _H_IRISLOOPSTATEMENT
#define _H_IRISLOOPSTATEMENT
#include "IrisStatement.h"

class IrisExpression;
class IrisIdentifier;
class IrisBlock;
class IrisLoopIfStatement :
	public IrisStatement
{
protected:
	IrisExpression* m_pCondition = nullptr;
	IrisExpression* m_pLoopTime = nullptr;
	IrisIdentifier* m_pLogVariable = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisLoopIfStatement(IrisExpression* pCondition, IrisExpression* pLoopTime, IrisIdentifier* pLogVariable, IrisBlock* pBlock);
	virtual ~IrisLoopIfStatement();
};

#endif;